use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Channels {
    pub inverter_status: lxp::status::StatusStore,
    pub from_inverter: broadcast::Sender<lxp::inverter::ChannelData>,
    pub to_inverter: broadcast::Sender<lxp::inverter::ChannelData>,
    pub from_mqtt: broadcast::Sender<mqtt::ChannelData>,
    pub to_mqtt: broadcast::Sender<mqtt::ChannelData>,
    pub to_influx: broadcast::Sender<influx::ChannelData>,
    pub to_database: broadcast::Sender<database::ChannelData>,
}

impl Default for Channels {
    fn default() -> Self {
        Self::new()
    }
}

impl Channels {
    /// Telemetry queues are bounded. Losing old samples must not kill a consumer.
    /// In-flight reply waits do not use this: a lost reply fails the command.
    pub async fn recv<T: Clone>(receiver: &mut broadcast::Receiver<T>) -> Result<T> {
        loop {
            match receiver.recv().await {
                Ok(data) => return Ok(data),
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    warn!(
                        "{} queue overflow: skipped {} old messages",
                        std::any::type_name::<T>(),
                        count
                    );
                }
                Err(broadcast::error::RecvError::Closed) => bail!("channel closed"),
            }
        }
    }

    pub fn new() -> Self {
        Self {
            inverter_status: lxp::status::StatusStore::new(),
            from_inverter: Self::channel(),
            to_inverter: Self::channel(),
            from_mqtt: Self::channel(),
            to_mqtt: Self::channel(),
            to_influx: Self::channel(),
            to_database: Self::channel(),
        }
    }

    fn channel<T: Clone>() -> broadcast::Sender<T> {
        broadcast::channel(2048).0 // we only need tx half
    }
}
