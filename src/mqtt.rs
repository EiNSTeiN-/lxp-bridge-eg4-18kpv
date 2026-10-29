use crate::prelude::*;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, LastWill, MqttOptions, Publish, QoS};

// Message {{{
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub retain: bool,
    pub payload: String,
}

pub enum TargetInverter {
    Serial(Serial),
    All,
}

impl Message {
    pub fn for_param(rp: lxp::packet::ReadParam) -> Result<Vec<Message>> {
        let mut r = Vec::new();

        for (register, value) in rp.pairs() {
            r.push(mqtt::Message {
                topic: format!("{}/param/{}", rp.datalog, register),
                retain: true,
                payload: serde_json::to_string(&value)?,
            });
        }

        Ok(r)
    }

    pub fn for_hold(td: lxp::packet::TranslatedData) -> Result<Vec<Message>> {
        let mut r = Vec::new();

        for (register, value) in td.pairs() {
            let mut scaled_value = value as f64;
            let config = lxp::packet::find_register_config(register as u16);

            if !config.is_none() {
                scaled_value = scaled_value * config.unwrap().scale;
            }

            r.push(mqtt::Message {
                topic: format!("{}/hold/{}", td.datalog, register),
                retain: true,
                payload: serde_json::to_string(&scaled_value)?,
            });

            if register == 21 {
                let bits = lxp::packet::Register21Bits::new(value);
                r.push(mqtt::Message {
                    topic: format!("{}/hold/{}/bits", td.datalog, register),
                    retain: true,
                    payload: serde_json::to_string(&bits)?,
                });
            }

            if register == 110 {
                let bits = lxp::packet::Register110Bits::new(value);
                r.push(mqtt::Message {
                    topic: format!("{}/hold/{}/bits", td.datalog, register),
                    retain: true,
                    payload: serde_json::to_string(&bits)?,
                });
            }
        }

        Ok(r)
    }

    pub fn for_input_all(
        inputs: &lxp::packet::ReadInputAll,
        datalog: lxp::inverter::Serial,
    ) -> Result<Message> {
        Ok(mqtt::Message {
            topic: format!("{}/inputs/all", datalog),
            retain: false,
            payload: serde_json::to_string(&inputs)?,
        })
    }

    pub fn for_input(
        td: lxp::packet::TranslatedData,
        publish_individual: bool,
    ) -> Result<Vec<Message>> {
        use lxp::packet::ReadInput;

        let mut r = Vec::new();

        let mut fault_code_registers_seen = false;
        let mut fault_code = 0;
        let mut warning_code_registers_seen = false;
        let mut warning_code = 0;

        for (register, value) in td.pairs() {
            if register == 0 {
                r.push(mqtt::Message {
                    topic: format!("{}/input/status/parsed", td.datalog),
                    retain: false,
                    payload: serde_json::to_string(lxp::packet::StatusString::from_value(value))?,
                });
            }

            if register == 71 {
                let bits = lxp::packet::Input71Bits::new(value);
                r.push(mqtt::Message {
                    topic: format!("{}/input/register_71/parsed", td.datalog),
                    retain: false,
                    payload: serde_json::to_string(&bits)?,
                });
            }

            if register == 77 {
                let bits = lxp::packet::Input77Bits::new(value);
                r.push(mqtt::Message {
                    topic: format!("{}/input/register_77/parsed", td.datalog),
                    retain: false,
                    payload: serde_json::to_string(&bits)?,
                });
            }

            if register == 113 {
                let bits = lxp::packet::Input113Bits::new(value);
                r.push(mqtt::Message {
                    topic: format!("{}/input/register_113/parsed", td.datalog),
                    retain: false,
                    payload: serde_json::to_string(&bits)?,
                });
            }

            if register == 144 {
                let bits = lxp::packet::Input144Bits::new(value);
                r.push(mqtt::Message {
                    topic: format!("{}/input/register_144/parsed", td.datalog),
                    retain: false,
                    payload: serde_json::to_string(&bits)?,
                });
            }

            if register == 60 {
                fault_code |= value as u32;
                fault_code_registers_seen = true;
            }
            if register == 61 {
                fault_code |= (value as u32) << 16;
                fault_code_registers_seen = true;
            }

            if register == 62 {
                warning_code |= value as u32;
                warning_code_registers_seen = true;
            }
            if register == 63 {
                warning_code |= (value as u32) << 16;
                warning_code_registers_seen = true;
            }
        }

        if warning_code_registers_seen {
            r.push(mqtt::Message {
                topic: format!("{}/input/warning_code/parsed", td.datalog),
                retain: false,
                payload: serde_json::to_string(lxp::packet::WarningCodeString::from_value(warning_code))?,
            });
        }

        if fault_code_registers_seen {
            r.push(mqtt::Message {
                topic: format!("{}/input/fault_code/parsed", td.datalog),
                retain: false,
                payload: serde_json::to_string(lxp::packet::FaultCodeString::from_value(fault_code))?,
            });
        }

        match td.read_input() {
            Ok(ReadInput::ReadInputAll(r_all)) => {
                // if publish_individual {
                    let mut data = serde_json::to_value(&r_all).unwrap();
                    for (key, value) in data.as_object_mut().unwrap() {
                        if key != "status" && key != "fault_code" && key != "warning_code"
                        && key != "register_71" && key != "register_77" && key != "register_113" {
                            r.push(mqtt::Message {
                                topic: format!("{}/input/{}/parsed", td.datalog, key),
                                retain: false,
                                payload: value.to_string(),
                            });
                        }
                    }
                // }
                // r.push(mqtt::Message {
                //     topic: format!("{}/inputs/all", td.datalog),
                //     retain: false,
                //     payload: serde_json::to_string(&r_all)?,
                // });
            },
            Ok(ReadInput::ReadInputAll2(r_all)) => {
                // if publish_individual {
                    let mut data = serde_json::to_value(&r_all).unwrap();
                    for (key, value) in data.as_object_mut().unwrap() {
                        if key != "register_144" {
                            r.push(mqtt::Message {
                                topic: format!("{}/input/{}/parsed", td.datalog, key),
                                retain: false,
                                payload: value.to_string(),
                            });
                        }
                    }
                // }
                // r.push(mqtt::Message {
                //     topic: format!("{}/inputs/all2", td.datalog),
                //     retain: false,
                //     payload: serde_json::to_string(&r_all)?,
                // });
            },
            Ok(ReadInput::ReadInput1(r1)) => r.push(mqtt::Message {
                topic: format!("{}/inputs/1", td.datalog),
                retain: false,
                payload: serde_json::to_string(&r1)?,
            }),
            Ok(ReadInput::ReadInput2(r2)) => r.push(mqtt::Message {
                topic: format!("{}/inputs/2", td.datalog),
                retain: false,
                payload: serde_json::to_string(&r2)?,
            }),
            Ok(ReadInput::ReadInput3(r3)) => r.push(mqtt::Message {
                topic: format!("{}/inputs/3", td.datalog),
                retain: false,
                payload: serde_json::to_string(&r3)?,
            }),
            Err(x) => warn!("ignoring {:?}", x),
        }

        Ok(r)
    }

    pub fn to_command(&self, inverter: config::Inverter) -> Result<Command> {
        use Command::*;

        let (_datalog, parts) = self.split_cmd_topic()?;

        let r = match parts[..] {
            ["read", "inputs", "1"] => ReadInputs(inverter, 1),
            ["read", "inputs", "2"] => ReadInputs(inverter, 2),
            ["read", "inputs", "3"] => ReadInputs(inverter, 3),
            ["read", "input", register] => {
                ReadInput(inverter, register.parse()?, self.payload_int_or_1()?)
            }
            ["read", "hold", register] => {
                ReadHold(inverter, register.parse()?, self.payload_int_or_1()?)
            }
            ["read", "param", register] => ReadParam(inverter, register.parse()?),
            ["read", "ac_charge", num] => ReadAcChargeTime(inverter, num.parse()?),
            ["read", "ac_first", num] => ReadAcFirstTime(inverter, num.parse()?),
            ["read", "charge_priority", num] => ReadChargePriorityTime(inverter, num.parse()?),
            ["read", "forced_discharge", num] => ReadForcedDischargeTime(inverter, num.parse()?),
            ["set", "hold", register] => {
                SetHold(inverter, register.parse()?, self.payload_float()?)
            }
            ["set", "param", register] => {
                WriteParam(inverter, register.parse()?, self.payload_int()?)
            }
            ["set", "ac_charge"] => AcCharge(inverter, self.payload_bool()?),
            ["set", "ac_charge", num] => {
                SetAcChargeTime(inverter, num.parse()?, self.payload_start_end_time()?)
            }
            ["set", "ac_first", num] => {
                SetAcFirstTime(inverter, num.parse()?, self.payload_start_end_time()?)
            }
            ["set", "charge_priority"] => ChargePriority(inverter, self.payload_bool()?),
            ["set", "charge_priority", num] => {
                SetChargePriorityTime(inverter, num.parse()?, self.payload_start_end_time()?)
            }
            ["set", "forced_discharge"] => ForcedDischarge(inverter, self.payload_bool()?),
            ["set", "forced_discharge", num] => {
                SetForcedDischargeTime(inverter, num.parse()?, self.payload_start_end_time()?)
            }
            ["set", "charge_rate_pct"] => ChargeRate(inverter, self.payload_percent()?),
            ["set", "discharge_rate_pct"] => DischargeRate(inverter, self.payload_percent()?),
            ["set", "ac_charge_rate_pct"] => AcChargeRate(inverter, self.payload_percent()?),

            ["set", "ac_charge_soc_limit_pct"] => AcChargeSocLimit(inverter, self.payload_percent()?),

            ["set", "discharge_cutoff_soc_limit_pct"] => {
                DischargeCutoffSocLimit(inverter, self.payload_percent()?)
            }

            [..] => bail!("unhandled: {:?}", self),
        };

        if let ReadInput(_, register, count) | ReadHold(_, register, count) = &r {
            if *count == 0 || *count > 127 || *register as u32 + *count as u32 > 65536 {
                bail!("invalid register read range");
            }
        }
        Ok(r)
    }

    // given a cmd Message, return the datalog it is intended for.
    //
    // eg cmd/AB12345678/set/ac_charge => (AB12345678, ['set', 'ac_charge'])
    pub fn split_cmd_topic(&self) -> Result<(TargetInverter, Vec<&str>)> {
        let parts: Vec<&str> = self.topic.split('/').collect();

        // bail if the topic is too short to handle.
        // this *shouldn't* happen as our subscribe is for lxp/cmd/{datalog}/#
        if parts.len() < 3 || parts[0] != "cmd" {
            bail!("ignoring badly formed MQTT topic: {}", self.topic);
        }

        // parts[0] should be cmd
        let datalog = parts[1];
        let rest = parts[2..].to_vec();

        if datalog == "all" {
            Ok((TargetInverter::All, rest))
        } else {
            let serial = Serial::from_str(datalog)?;
            Ok((TargetInverter::Serial(serial), rest))
        }
    }

    // not entirely happy with this return type but it avoids needing to expose a struct for now
    fn payload_start_end_time(&self) -> Result<[u8; 4]> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct StartEndTime {
            start: String,
            end: String,
        }

        // {"start":"20:00", "end":"21:00"} -> [20, 0, 21, 0]
        let t = serde_json::from_str::<StartEndTime>(&self.payload)?;
        // split on : then make u8s to return in an array
        let start: Vec<&str> = t.start.split(':').collect();
        let end: Vec<&str> = t.end.split(':').collect();
        if start.len() != 2 || end.len() != 2 {
            bail!("badly formatted time, use HH:MM")
        }
        let values = [
            start[0].parse()?, start[1].parse()?,
            end[0].parse()?, end[1].parse()?,
        ];
        if values[0] > 23 || values[2] > 23 || values[1] > 59 || values[3] > 59 {
            bail!("time out of range, use HH:MM");
        }
        Ok(values)
    }

    fn payload_int_or_1(&self) -> Result<u16> {
        if self.payload.is_empty() {
            Ok(1)
        } else {
            self.payload_int()
        }
    }

    fn payload_int(&self) -> Result<u16> {
        self.payload
            .parse()
            .map_err(|err| anyhow!("payload_int: {}", err))
    }

    fn payload_percent(&self) -> Result<u16> {
        let value = self.payload_int()?;
        if value > 100 {
            bail!("percentage must be between 0 and 100");
        }
        Ok(value)
    }

    fn payload_float(&self) -> Result<f64> {
        let value: f64 = self.payload.parse()
            .map_err(|err| anyhow!("payload_float: {}", err))?;
        if !value.is_finite() {
            bail!("value must be finite");
        }
        Ok(value)
    }

    fn payload_bool(&self) -> Result<bool> {
        match self.payload.to_ascii_lowercase().as_str() {
            "1" | "t" | "true" | "on" | "y" | "yes" => Ok(true),
            "0" | "f" | "false" | "off" | "n" | "no" => Ok(false),
            _ => bail!("invalid boolean payload"),
        }
    }
} // }}}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ChannelData {
    Message(Message),
    Shutdown,
}

pub type Sender = broadcast::Sender<ChannelData>;

pub struct Mqtt {
    config: ConfigWrapper,
    shutdown: bool,
    channels: Channels,
}

impl Mqtt {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        Self {
            config,
            channels,
            shutdown: false,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let c = &self.config;

        if !c.mqtt().enabled() {
            info!("mqtt disabled, skipping");
            return Ok(());
        }

        let mut options = MqttOptions::new("lxp-bridge", c.mqtt().host(), c.mqtt().port());

        let will = LastWill {
            topic: self.lwt_topic(),
            message: bytes::Bytes::from("offline"),
            qos: QoS::AtLeastOnce,
            retain: true,
        };
        options.set_last_will(will);

        options.set_keep_alive(std::time::Duration::from_secs(60));
        if let (Some(u), Some(p)) = (c.mqtt().username(), c.mqtt().password()) {
            options.set_credentials(u, p);
        }

        info!(
            "initializing mqtt at {}:{}",
            c.mqtt().host(),
            c.mqtt().port()
        );

        for inverter in c.inverters().iter() {
            self.channels.inverter_status.register(inverter);
        }

        let (client, eventloop) = AsyncClient::new(options, 10);

        let (connected, mut connections) = tokio::sync::mpsc::channel(1);
        let (ready, status_ready) = tokio::sync::watch::channel(false);
        let setup = async {
            while connections.recv().await.is_some() {
                // Reannounce online status, subscriptions, and discovery on every CONNACK.
                // Run beside poll(), since publishing discovery can fill the client queue.
                self.setup(client.clone()).await?;
                // A single publisher serializes status updates and replays the
                // latest in-memory state after discovery on each broker connection.
                let _ = ready.send(true);
            }
            Ok::<(), anyhow::Error>(())
        };
        futures::try_join!(
            setup,
            self.receiver(eventloop, connected),
            self.sender(client.clone()),
            self.status_sender(client.clone(), status_ready)
        )?;

        Ok(())
    }

    pub fn stop(&mut self) {
        self.shutdown = true;

        let _ = self.channels.from_mqtt.send(ChannelData::Shutdown);
    }

    async fn setup(&self, client: AsyncClient) -> Result<()> {
        client
            .subscribe(
                format!("{}/cmd/all/#", self.config.mqtt().namespace()),
                QoS::AtMostOnce,
            )
            .await?;

        for inverter in self.config.enabled_inverters() {
            client
                .subscribe(
                    format!(
                        "{}/cmd/{}/#",
                        self.config.mqtt().namespace(),
                        inverter.datalog()
                    ),
                    QoS::AtMostOnce,
                )
                .await?;

            if self.config.mqtt().homeassistant().enabled() {
                let ha = home_assistant::Config::new(&inverter, &self.config.mqtt());
                for msg in ha.all()?.into_iter() {
                    let _ = client
                        .publish(&msg.topic, QoS::AtLeastOnce, msg.retain, msg.payload)
                        .await;
                }
            }
        }

        if self.config.mqtt().homeassistant().enabled() {
            for inverter in self.config.inverters().iter().filter(|inverter| !inverter.enabled()) {
                let message = home_assistant::Config::new(inverter, &self.config.mqtt()).connection_status()?;
                client.publish(message.topic, QoS::AtLeastOnce, true, message.payload).await?;
            }
        }

        Ok(())
    }

    async fn status_sender(&self, client: AsyncClient, mut ready: tokio::sync::watch::Receiver<bool>) -> Result<()> {
        let mut statuses = self.channels.inverter_status.subscribe();
        let mut published = lxp::status::Snapshot::new();
        loop {
            let replay = tokio::select! {
                result = ready.changed() => { result?; true },
                result = statuses.changed() => { result?; false },
            };
            if !*ready.borrow() { continue; }
            let current = statuses.borrow_and_update().clone();
            for (datalog, status) in &current {
                if replay || published.get(datalog) != Some(status) {
                    let topic = format!("{}/{}/status", self.config.mqtt().namespace(), datalog);
                    client.publish(topic, QoS::AtLeastOnce, true, serde_json::to_vec(status)?).await?;
                }
            }
            // Publish the current status before marking the bridge online, so a
            // fresh process replaces retained statuses from its predecessor.
            if replay {
                client.publish(self.lwt_topic(), QoS::AtLeastOnce, true, "online").await?;
            }
            published = current;
        }
    }

    // mqtt -> coordinator
    async fn receiver(
        &self,
        mut eventloop: EventLoop,
        connected: tokio::sync::mpsc::Sender<()>,
    ) -> Result<()> {
        loop {
            if self.shutdown {
                break;
            }
            match eventloop.poll().await {
                Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                    let _ = connected.try_send(());
                }
                Ok(Event::Incoming(Incoming::Publish(publish))) => {
                    if let Err(error) = self.handle_message(publish) {
                        warn!("ignoring invalid MQTT message: {}", error);
                    }
                }
                Err(error) => {
                    error!("MQTT: {} - reconnecting in 5s", error);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_message(&self, publish: Publish) -> Result<()> {
        // remove the namespace, including the first /
        // doing it this way means we don't break if namespace happens to contain a /
        let prefix = format!("{}/", self.config.mqtt().namespace());
        let topic = publish
            .topic
            .strip_prefix(&prefix)
            .ok_or_else(|| anyhow!("topic outside configured namespace"))?
            .to_owned();

        let message = Message {
            topic,
            retain: publish.retain,
            payload: String::from_utf8(publish.payload.to_vec())?,
        };
        debug!("RX: {:?}", message);
        if self
            .channels
            .from_mqtt
            .send(ChannelData::Message(message))
            .is_err()
        {
            bail!("send(from_mqtt) failed - channel closed?");
        }

        Ok(())
    }

    // coordinator -> mqtt
    async fn sender(&self, client: AsyncClient) -> Result<()> {
        use ChannelData::*;

        let mut receiver = self.channels.to_mqtt.subscribe();

        loop {
            match Channels::recv(&mut receiver).await? {
                Shutdown => break,
                Message(message) => {
                    let topic = format!("{}/{}", self.config.mqtt().namespace(), message.topic);
                    info!("publishing: {} = {}", topic, message.payload);
                    let _ = client
                        .publish(&topic, QoS::AtLeastOnce, message.retain, message.payload)
                        .await
                        .map_err(|err| error!("publish {} failed: {:?} .. skipping", topic, err));
                }
            }
        }

        info!("sender loop exiting");

        Ok(())
    }

    fn lwt_topic(&self) -> String {
        format!("{}/LWT", self.config.mqtt().namespace())
    }
}
