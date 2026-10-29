use crate::prelude::*;

use chrono::TimeZone;
use rinfluxdb::line_protocol::{r#async::Client, LineBuilder};

static INPUTS_MEASUREMENT: &str = "inputs";

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ChannelData {
    InputData(serde_json::Value),
    Shutdown,
}

pub type Sender = broadcast::Sender<ChannelData>;

pub struct Influx {
    config: ConfigWrapper,
    channels: Channels,
}

impl Influx {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        Self { config, channels }
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.influx().enabled() {
            info!("influx disabled, skipping");
            return Ok(());
        }

        info!("initializing influx at {}", self.config.influx().url());

        let client = {
            let config = self.config.influx();
            let url = reqwest::Url::parse(config.url())?;
            let credentials = match (config.username(), config.password()) {
                (Some(u), Some(p)) => Some((u, p)),
                _ => None,
            };

            Client::new(url, credentials)?
        };

        let mut receiver = self.channels.to_influx.subscribe();
        let mut shutdown = self.channels.to_influx.subscribe();
        tokio::select! {
            result = self.sender(client, &mut receiver) => result?,
            _ = async {
                while let Ok(data) = Channels::recv(&mut shutdown).await {
                    if data == ChannelData::Shutdown { break; }
                }
            } => {},
        }

        info!("influx loop exiting");

        Ok(())
    }

    pub fn stop(&self) {
        let _ = self.channels.to_influx.send(ChannelData::Shutdown);
    }

    async fn sender(
        &self,
        client: Client,
        receiver: &mut broadcast::Receiver<ChannelData>,
    ) -> Result<()> {
        loop {
            match Channels::recv(receiver).await? {
                ChannelData::Shutdown => break,
                ChannelData::InputData(data) => {
                    if let Err(error) = self.send_data(&client, data).await {
                        warn!("skipping invalid InfluxDB sample: {}", error);
                    }
                }
            }
        }
        Ok(())
    }

    async fn send_data(&self, client: &Client, data: serde_json::Value) -> Result<()> {
        let mut line = LineBuilder::new(INPUTS_MEASUREMENT);
        for (key, value) in data
            .as_object()
            .ok_or_else(|| anyhow!("expected an input object"))?
        {
            let key = key.to_string();
            line = if key == "time" {
                let value = value.as_i64().ok_or_else(|| anyhow!("invalid timestamp"))?;
                let time = chrono::Utc
                    .timestamp_opt(value, 0)
                    .single()
                    .ok_or_else(|| anyhow!("timestamp out of range"))?;
                line.set_timestamp(time)
            } else if key == "datalog" {
                line.insert_tag(
                    key,
                    value.as_str().ok_or_else(|| anyhow!("invalid datalog"))?,
                )
            } else if let Some(value) = value.as_i64() {
                line.insert_field(key, value)
            } else if let Some(value) = value.as_f64() {
                line.insert_field(key, value)
            } else {
                bail!("non-numeric input field {}", key);
            };
        }
        let lines = vec![line.build()];
        loop {
            match tokio::time::timeout(
                std::time::Duration::from_secs(30),
                client.send(&self.database(), &lines),
            )
            .await
            {
                Ok(Ok(_)) => return Ok(()),
                Ok(Err(error)) => error!("InfluxDB push failed: {:?} - retrying in 10s", error),
                Err(_) => error!("InfluxDB push timed out - retrying in 10s"),
            }
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }

    fn database(&self) -> String {
        self.config.influx().database().to_string()
    }
}
