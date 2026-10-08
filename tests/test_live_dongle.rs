// Explicitly enabled, read-only hardware test. Never runs in normal CI.
use lxp_bridge::prelude::*;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

async fn read_registers<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    datalog: Serial,
    inverter: Serial,
    function: lxp::packet::DeviceFunction,
) -> Result<()> {
    let request = Packet::TranslatedData(lxp::packet::TranslatedData {
        datalog,
        inverter,
        device_function: function,
        register: 0,
        values: 40_u16.to_le_bytes().to_vec(),
    });
    stream
        .write_all(&lxp::packet::TcpFrameFactory::build(&request))
        .await?;
    loop {
        let mut header = [0; 6];
        stream.read_exact(&mut header).await?;
        let length = usize::from(u16::from_le_bytes([header[4], header[5]]));
        assert!(length <= 4096, "unexpected frame size");
        let mut frame = header.to_vec();
        frame.resize(6 + length, 0);
        stream.read_exact(&mut frame[6..]).await?;
        match lxp::packet::Parser::parse(&frame)? {
            Packet::Heartbeat(heartbeat) => {
                stream
                    .write_all(&lxp::packet::TcpFrameFactory::build(&Packet::Heartbeat(
                        heartbeat,
                    )))
                    .await?;
            }
            Packet::TranslatedData(reply) if reply.device_function == function => {
                assert_eq!(reply.datalog, datalog);
                assert_eq!(reply.inverter, inverter);
                assert_eq!(reply.register, 0);
                assert!(reply.values.len() >= 80);
                if function == lxp::packet::DeviceFunction::ReadInput {
                    reply.read_input()?;
                }
                println!(
                    "{:?}: {} frame bytes, {} registers; decoded successfully",
                    function,
                    frame.len(),
                    reply.values.len() / 2
                );
                return Ok(());
            }
            _ => {}
        }
    }
}

async fn check<S: AsyncRead + AsyncWrite + Unpin>(
    mut stream: S,
    datalog: Serial,
    inverter: Serial,
) -> Result<()> {
    for function in [
        lxp::packet::DeviceFunction::ReadHold,
        lxp::packet::DeviceFunction::ReadInput,
    ] {
        tokio::time::timeout(
            Duration::from_secs(10),
            read_registers(&mut stream, datalog, inverter, function),
        )
        .await??;
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires LXP_LIVE_HOST, LXP_LIVE_DATALOG, LXP_LIVE_SERIAL and LXP_LIVE_TLS"]
async fn read_only_live_dongle() -> Result<()> {
    let host = std::env::var("LXP_LIVE_HOST")?;
    let datalog = std::env::var("LXP_LIVE_DATALOG")?.parse()?;
    let inverter = std::env::var("LXP_LIVE_SERIAL")?.parse()?;
    let stream = tokio::net::TcpStream::connect(host).await?;
    if std::env::var("LXP_LIVE_TLS")? == "true" {
        check(lxp::tls::connect(stream, datalog).await?, datalog, inverter).await
    } else {
        check(stream, datalog, inverter).await
    }
}
