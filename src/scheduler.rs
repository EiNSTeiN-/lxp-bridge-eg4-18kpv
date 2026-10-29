use crate::prelude::*;

use cron_parser::parse;

use chrono::{DateTime, Local};

pub struct Scheduler {
    config: ConfigWrapper,
    channels: Channels,
}

impl Scheduler {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        Self { config, channels }
    }

    pub async fn start(&self) -> Result<()> {
        let scheduler = self.config.scheduler().clone();
        if scheduler.is_none() {
            info!("scheduler config not found, skipping");
            return Ok(());
        }

        let scheduler = scheduler.unwrap();
        if !scheduler.enabled() {
            info!("scheduler disabled, skipping");
            return Ok(());
        }

        info!("scheduler starting");

        if let Some(timesync_cron) = scheduler.timesync_cron() {
            // sticking to Utc here avoids some "invalid date" panics around DST changes
            loop {
                let next = parse(timesync_cron, &Utils::utc())
                    .map_err(|error| anyhow!("invalid timesync cron: {}", error))?;
                let sleep = next - Utils::utc();

                // localtime is only used for display
                let local_next: DateTime<Local> = DateTime::from(next);
                info!("next timesync at {}, sleeping for {}", local_next, sleep);

                tokio::time::sleep(sleep.to_std().unwrap_or_default()).await;
                self.timesync().await?;
            }
        } else {
            info!("timesync_cron config not found, skipping");
        }

        info!("scheduler exiting");

        Ok(())
    }

    async fn timesync(&self) -> Result<()> {
        info!("timesync starting");

        let syncs = self
            .config
            .enabled_inverters()
            .into_iter()
            .map(|inverter| async move {
                let datalog = inverter.datalog();
                if let Err(error) =
                    coordinator::commands::timesync::TimeSync::new(self.channels.clone(), inverter)
                        .run()
                        .await
                {
                    warn!(
                        "inverter {} timesync failed: {:#}; will retry at next scheduled run",
                        datalog, error
                    );
                }
            });
        futures::future::join_all(syncs).await;

        info!("timesync complete");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn failed_timesync_does_not_skip_other_inverters_or_future_runs() {
        let config = ConfigWrapper::new("config.yaml.example".into()).unwrap();
        let first = config.enabled_inverters()[0].clone();
        let mut second = first.clone();
        second.datalog = "3333333333".parse().unwrap();
        config.set_inverters(vec![first.clone(), second.clone()]);
        let channels = Channels::new();
        let scheduler = Scheduler::new(config, channels.clone());
        let mut outgoing = channels.to_inverter.subscribe();
        for _ in 0..2 {
            let replies = async {
                let mut targets = Vec::new();
                for _ in 0..2 {
                    let request = outgoing.recv().await.unwrap();
                    if let lxp::inverter::ChannelData::Packet(packet) = request {
                        targets.push(packet.datalog());
                        channels
                            .from_inverter
                            .send(lxp::inverter::ChannelData::Disconnect(packet.datalog()))
                            .unwrap();
                    }
                }
                assert!(targets.contains(&first.datalog()));
                assert!(targets.contains(&second.datalog()));
            };
            let (result, ()) = tokio::time::timeout(std::time::Duration::from_millis(500), async {
                tokio::join!(scheduler.timesync(), replies)
            })
            .await
            .unwrap();
            result.unwrap();
        }
    }
}
