use crate::prelude::*;

use {
    async_trait::async_trait,
    serde::{Serialize, Serializer},
    tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ChannelData {
    Connected(Serial),  // strictly speaking, these two only ever go
    Disconnect(Serial), // inverter->coordinator, but eh.
    Packet(Packet),     // this one goes both ways through the channel.
    Shutdown,
}
pub type Sender = broadcast::Sender<ChannelData>;
pub type Receiver = broadcast::Receiver<ChannelData>;

// WaitForReply {{{
#[async_trait]
pub trait WaitForReply {
    #[cfg(not(feature = "mocks"))]
    const TIMEOUT: u64 = 10;

    #[cfg(feature = "mocks")]
    const TIMEOUT: u64 = 1; // fail quickly in tests

    async fn wait_for_reply(&mut self, packet: &Packet) -> Result<Packet>;
}
#[async_trait]
impl WaitForReply for Receiver {
    async fn wait_for_reply(&mut self, packet: &Packet) -> Result<Packet> {
        tokio::time::timeout(std::time::Duration::from_secs(Self::TIMEOUT), async {
            loop {
                match (packet, self.recv().await) {
                    (
                        Packet::TranslatedData(td),
                        Ok(ChannelData::Packet(Packet::TranslatedData(reply))),
                    ) => {
                        if td.datalog == reply.datalog
                            && td.register == reply.register
                            && td.device_function == reply.device_function
                            && td.inverter == reply.inverter
                        {
                            return Ok(Packet::TranslatedData(reply));
                        }
                    }
                    (Packet::ReadParam(rp), Ok(ChannelData::Packet(Packet::ReadParam(reply)))) => {
                        if rp.datalog == reply.datalog && rp.register == reply.register {
                            return Ok(Packet::ReadParam(reply));
                        }
                    }
                    (
                        Packet::WriteParam(wp),
                        Ok(ChannelData::Packet(Packet::WriteParam(reply))),
                    ) => {
                        if wp.datalog == reply.datalog && wp.register == reply.register {
                            return Ok(Packet::WriteParam(reply));
                        }
                    }
                    (_, Ok(ChannelData::Packet(_))) => {} // TODO ReadParam and WriteParam
                    (_, Ok(ChannelData::Connected(_))) => {} // Breaks on channel overflow, but we have a timeout
                    (_, Ok(ChannelData::Disconnect(inverter_datalog))) => {
                        if inverter_datalog == packet.datalog() {
                            bail!("inverter {} disconnected", inverter_datalog);
                        }
                    }
                    (_, Ok(ChannelData::Shutdown)) => bail!("shutting down"),
                    (_, Err(err)) => bail!("reply channel error: {:?}", err),
                }
            }
        })
        .await
        .map_err(|_| anyhow!("wait_for_reply {:?} - timeout", packet))?
    }
} // }}}

// Serial {{{
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Serial([u8; 10]);

impl Serial {
    pub fn new(input: &[u8]) -> Result<Self> {
        Ok(Self(input.try_into()?))
    }

    pub fn default() -> Self {
        Self([0; 10])
    }

    pub fn data(&self) -> [u8; 10] {
        self.0
    }
}

impl Serialize for Serial {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl std::str::FromStr for Serial {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 10 {
            return Err(anyhow!("{} must be exactly 10 characters", s));
        }

        let mut r: [u8; 10] = Default::default();
        r.copy_from_slice(s.as_bytes());
        Ok(Self(r))
    }
}

impl std::fmt::Display for Serial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

impl std::fmt::Debug for Serial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
} // }}}

pub struct Inverter {
    config: ConfigWrapper,
    datalog: Serial,
    channels: Channels,
}

impl Inverter {
    pub fn new(config: ConfigWrapper, inverter: &config::Inverter, channels: Channels) -> Self {
        // remember which inverter this instance is for
        let datalog = inverter.datalog();
        channels.inverter_status.register(inverter);

        Self {
            config,
            datalog,
            channels,
        }
    }

    pub fn config(&self) -> config::Inverter {
        self.config
            .inverters()
            .iter()
            .find(|inverter| inverter.datalog() == self.datalog)
            .cloned()
            .expect("can't find my inverter")
    }

    pub async fn start(&self) -> Result<()> {
        // Listen throughout TCP connect, TLS negotiation, and reconnect backoff.
        let mut shutdown = self.channels.to_inverter.subscribe();
        let result = tokio::select! {
            result = self.reconnect() => result,
            _ = async {
                while let Ok(data) = Channels::recv(&mut shutdown).await {
                    if data == ChannelData::Shutdown { break; }
                }
            } => Ok(()),
        };
        self.channels
            .inverter_status
            .disconnected(self.datalog, None);
        result
    }

    async fn reconnect(&self) -> Result<()> {
        loop {
            self.channels.inverter_status.connecting(self.datalog);
            let e = match self.connect().await {
                Ok(()) => return Ok(()),
                Err(error) => error,
            };
            self.channels
                .inverter_status
                .disconnected(self.datalog, Some(format!("{:#}", e)));
            error!("inverter {}: {}", self.config().datalog(), e);
            info!("inverter {}: reconnecting in 5s", self.config().datalog());
            let _ = self
                .channels
                .from_inverter
                .send(ChannelData::Disconnect(self.config().datalog())); // kill any waiting readers
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    pub fn stop(&self) {
        let _ = self.channels.to_inverter.send(ChannelData::Shutdown);
    }

    async fn connect(&self) -> Result<()> {
        use net2::TcpStreamExt; // for set_keepalive

        info!(
            "connecting to inverter {} at {}:{}",
            self.config().datalog(),
            self.config().host(),
            self.config().port()
        );

        let inverter_hp = (self.config().host().to_owned(), self.config().port());

        let stream = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::net::TcpStream::connect(inverter_hp),
        )
        .await
        .map_err(|_| anyhow!("TCP connection timed out after 10 seconds"))??;
        let std_stream = stream.into_std()?;
        std_stream.set_keepalive(Some(std::time::Duration::new(60, 0)))?;
        let stream = tokio::net::TcpStream::from_std(std_stream)?;

        if self.config().tls() {
            self.channels.inverter_status.authenticating(self.datalog);
            let stream = lxp::tls::connect(stream, self.config().datalog()).await?;
            self.run_connection(stream).await
        } else {
            self.run_connection(stream).await
        }
    }

    async fn run_connection<S>(&self, stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let (reader, writer) = tokio::io::split(stream);
        // Subscribe before announcing Connected so startup reads cannot be lost.
        let outgoing = self.channels.to_inverter.subscribe();

        info!("inverter {}: connected!", self.config().datalog());
        self.channels.inverter_status.connected(self.datalog);
        self.channels
            .from_inverter
            .send(ChannelData::Connected(self.config().datalog()))?;

        // Dropping the other half also closes TLS connections on shutdown/error.
        tokio::select! {
            result = self.sender(writer, outgoing) => result,
            result = self.receiver(reader) => result,
        }
    }

    // inverter -> coordinator
    async fn receiver<R: AsyncRead + Unpin>(&self, mut socket: R) -> Result<()> {
        use std::time::Duration;
        use tokio::time::timeout;
        use {bytes::BytesMut, tokio_util::codec::Decoder};

        let mut buf = BytesMut::new();
        let mut decoder = lxp::packet_decoder::PacketDecoder::new();

        loop {
            // read_buf appends to buf rather than overwrite existing data
            let future = socket.read_buf(&mut buf);
            let read_timeout = self.config().read_timeout();
            let len = if read_timeout > 0 {
                match timeout(Duration::from_secs(read_timeout), future).await {
                    Ok(r) => r,
                    Err(_) => bail!("no data for {} seconds", read_timeout),
                }
            } else {
                future.await
            }?;

            if len == 0 {
                while let Some(packet) = decoder.decode_eof(&mut buf)? {
                    self.handle_incoming_packet(packet)?;
                }
                break;
            }

            while let Some(packet) = decoder.decode(&mut buf)? {
                self.handle_incoming_packet(packet.clone())?;

                self.compare_datalog(packet.datalog()); // all packets have datalog serial
                if let Packet::TranslatedData(td) = packet {
                    // only TranslatedData has inverter serial
                    self.compare_inverter(td.inverter);
                };
            }
        }

        Err(anyhow!("lost connection"))
    }

    fn handle_incoming_packet(&self, packet: Packet) -> Result<()> {
        let is_input = matches!(&packet, Packet::TranslatedData(td)
            if td.device_function == lxp::packet::DeviceFunction::ReadInput);
        self.channels
            .inverter_status
            .packet_received(self.datalog, is_input);
        // bytes received are logged in packet_decoder, no need here
        //debug!("inverter {}: RX {:?}", self.config.datalog, packet);

        if self.config().heartbeats()
            && packet.tcp_function() == lxp::packet::TcpFunction::Heartbeat
        {
            self.channels
                .to_inverter
                .send(ChannelData::Packet(packet.clone()))?;
        }

        self.channels
            .from_inverter
            .send(ChannelData::Packet(packet))?;

        Ok(())
    }

    // coordinator -> inverter
    async fn sender<W: AsyncWrite + Unpin>(
        &self,
        mut socket: W,
        mut receiver: Receiver,
    ) -> Result<()> {
        use ChannelData::*;

        loop {
            match receiver.recv().await? {
                Shutdown => break,
                // this doesn't actually happen yet; (Dis)connect is never sent to this channel
                Connected(_) => {}
                Disconnect(serial) => {
                    if serial == self.config().datalog() {
                        bail!("sender exiting due to ChannelData::Disconnect");
                    }
                }
                Packet(packet) => {
                    // this works, but needs more thought. because we only fix it here, immediately
                    // before transmission, calls to wait_for_reply with the original serials will
                    // never complete. ideally we need to pass the fixed packet back?
                    //self.fix_outgoing_packet_serials(&mut packet);

                    if packet.datalog() == self.config().datalog() {
                        //debug!("inverter {}: TX {:?}", self.config.datalog, packet);
                        let bytes = lxp::packet::TcpFrameFactory::build(&packet);
                        debug!("inverter {}: TX {:?}", self.config().datalog(), bytes);
                        tokio::time::timeout(
                            std::time::Duration::from_secs(10),
                            socket.write_all(&bytes),
                        )
                        .await
                        .map_err(|_| anyhow!("socket write timed out after 10 seconds"))??;
                    }
                }
            }
        }

        info!("inverter {}: sender exiting", self.config().datalog());

        Ok(())
    }

    /* TODO. need to solve wait_for_reply hanging when we fix the serials.. */
    /*
    #[allow(dead_code)]
    fn fix_outgoing_packet_serials(&self, packet: &mut Packet) {
        let ob = self.serials.borrow();
        if packet.datalog() != ob.datalog {
            warn!(
                "fixing datalog in outgoing packet from {} to {}",
                packet.datalog(),
                ob.datalog
            );
            packet.set_datalog(ob.datalog);
        }

        if let Some(inverter) = packet.inverter() {
            if inverter != ob.inverter {
                warn!(
                    "fixing serial in outgoing packet from {} to {}",
                    inverter, ob.inverter
                );
                packet.set_inverter(ob.inverter);
            }
        }
    }
    */

    fn compare_datalog(&self, packet: Serial) {
        if packet != self.config().datalog() {
            warn!(
                "datalog serial mismatch found; packet={}, config={} - please check config!",
                packet,
                self.config().datalog()
            );
            // uncomment this when I fix serials in outgoing packets?
            //self.config.datalog = packet;
        }
    }

    fn compare_inverter(&self, packet: Serial) {
        if packet != self.config().serial() {
            warn!(
                "inverter serial mismatch found; packet={}, config={} - please check config!",
                packet,
                self.config().serial()
            );
            // uncomment this when I fix serials in outgoing packets?
            //self.config.serial = packet;
        }
    }
}
