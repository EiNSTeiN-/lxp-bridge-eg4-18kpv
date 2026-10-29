mod common;
use common::*;

use lxp::inverter::ChannelData as InverterData;
use lxp::packet::{DeviceFunction, TranslatedData};
use std::time::Duration;
use tokio::time::timeout;

fn two_inverters() -> (ConfigWrapper, config::Inverter, config::Inverter) {
    let config = Factory::example_config_wrapped();
    config.influx_mut().enabled = false;
    config.set_databases(vec![]);
    let mut slow = Factory::inverter();
    slow.publish_holdings_on_connect = Some(true);
    let mut healthy = slow.clone();
    healthy.datalog = "3333333333".parse().unwrap();
    healthy.serial = "6666666666".parse().unwrap();
    healthy.publish_holdings_on_connect = Some(false);
    config.set_inverters(vec![slow.clone(), healthy.clone()]);
    (config, slow, healthy)
}

fn holding(inverter: &config::Inverter, register: u16) -> Packet {
    Packet::TranslatedData(TranslatedData {
        datalog: inverter.datalog(),
        inverter: inverter.serial(),
        device_function: DeviceFunction::ReadHold,
        register,
        values: vec![22, 6],
    })
}

async fn next_message(receiver: &mut broadcast::Receiver<mqtt::ChannelData>) -> mqtt::Message {
    match timeout(Duration::from_millis(500), receiver.recv())
        .await
        .unwrap()
        .unwrap()
    {
        mqtt::ChannelData::Message(message) => message,
        other => panic!("unexpected message {:?}", other),
    }
}

#[tokio::test]
async fn initialization_wait_does_not_block_other_inverter_telemetry() {
    let (config, slow, healthy) = two_inverters();
    let channels = Channels::new();
    let coordinator = Coordinator::new(config, channels.clone());
    let mut outgoing = channels.to_inverter.subscribe();
    let mut mqtt = channels.to_mqtt.subscribe();
    let assertions = async {
        channels
            .from_inverter
            .send(InverterData::Connected(slow.datalog()))
            .unwrap();
        // Wait until initialization really is waiting for an unanswered request.
        let request = timeout(Duration::from_millis(500), outgoing.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            unwrap_inverter_channeldata_packet(request).datalog(),
            slow.datalog()
        );
        channels
            .from_inverter
            .send(InverterData::Packet(holding(&healthy, 12)))
            .unwrap();
        let message = next_message(&mut mqtt).await;
        assert_eq!(message.topic, format!("{}/hold/12", healthy.datalog()));
        // Reconnecting repeatedly must not stall telemetry or leak initializers.
        for _ in 0..3 {
            channels
                .from_inverter
                .send(InverterData::Disconnect(slow.datalog()))
                .unwrap();
            channels
                .from_inverter
                .send(InverterData::Connected(slow.datalog()))
                .unwrap();
            let _ = timeout(Duration::from_millis(500), outgoing.recv())
                .await
                .unwrap()
                .unwrap();
            channels
                .from_inverter
                .send(InverterData::Packet(holding(&healthy, 12)))
                .unwrap();
            assert_eq!(
                next_message(&mut mqtt).await.topic,
                format!("{}/hold/12", healthy.datalog())
            );
        }
        coordinator.stop();
    };
    let (result, ()) = tokio::join!(coordinator.start(), assertions);
    result.unwrap();
}

#[tokio::test]
async fn healthy_commands_continue_while_another_inverter_waits() {
    let (config, slow, healthy) = two_inverters();
    let channels = Channels::new();
    let coordinator = Coordinator::new(config, channels.clone());
    let mut outgoing = channels.to_inverter.subscribe();
    let mut mqtt = channels.to_mqtt.subscribe();
    let assertions = async {
        channels
            .from_mqtt
            .send(mqtt::ChannelData::Message(mqtt::Message {
                topic: "cmd/all/read/hold/12".into(),
                retain: false,
                payload: String::new(),
            }))
            .unwrap();
        let mut targets = Vec::new();
        for _ in 0..2 {
            let request = timeout(Duration::from_millis(500), outgoing.recv())
                .await
                .unwrap()
                .unwrap();
            targets.push(unwrap_inverter_channeldata_packet(request).datalog());
        }
        assert!(targets.contains(&slow.datalog()));
        assert!(targets.contains(&healthy.datalog()));
        channels
            .from_inverter
            .send(InverterData::Packet(holding(&healthy, 12)))
            .unwrap();
        loop {
            let message = next_message(&mut mqtt).await;
            if message.topic.starts_with("result/") {
                assert_eq!(
                    message.topic,
                    format!("result/{}/read/hold/12", healthy.datalog())
                );
                assert_eq!(message.payload, "OK");
                break;
            }
        }
        channels
            .from_mqtt
            .send(mqtt::ChannelData::Message(mqtt::Message {
                topic: format!("cmd/{}/read/hold/13", healthy.datalog()),
                retain: false,
                payload: String::new(),
            }))
            .unwrap();
        let request = timeout(Duration::from_millis(500), outgoing.recv())
            .await
            .unwrap()
            .unwrap();
        let packet = unwrap_inverter_channeldata_packet(request);
        assert_eq!(packet.datalog(), healthy.datalog());
        assert_eq!(packet.register(), 13);
        coordinator.stop();
    };
    let (result, ()) = tokio::join!(coordinator.start(), assertions);
    result.unwrap();
}

#[tokio::test]
async fn coordinator_recovers_after_broadcast_overflow() {
    let (config, _, healthy) = two_inverters();
    let channels = Channels::new();
    let coordinator = Coordinator::new(config, channels.clone());
    let mut mqtt = channels.to_mqtt.subscribe();
    let assertions = async {
        for _ in 0..3000 {
            channels
                .from_inverter
                .send(InverterData::Packet(Packet::Heartbeat(
                    lxp::packet::Heartbeat {
                        datalog: healthy.datalog(),
                    },
                )))
                .unwrap();
        }
        channels
            .from_inverter
            .send(InverterData::Packet(holding(&healthy, 12)))
            .unwrap();
        assert_eq!(
            next_message(&mut mqtt).await.topic,
            format!("{}/hold/12", healthy.datalog())
        );
        coordinator.stop();
    };
    let (result, ()) = tokio::join!(coordinator.start(), assertions);
    result.unwrap();
}

#[tokio::test]
async fn unavailable_storage_does_not_stop_mqtt_telemetry() {
    let (config, _, healthy) = two_inverters();
    config.influx_mut().enabled = true;
    config.set_databases(vec![config::Database {
        enabled: true,
        url: "sqlite::memory:".into(),
    }]);
    let channels = Channels::new();
    let coordinator = Coordinator::new(config, channels.clone());
    let mut mqtt = channels.to_mqtt.subscribe();
    let assertions = async {
        channels
            .from_inverter
            .send(InverterData::Packet(Packet::TranslatedData(
                TranslatedData {
                    datalog: healthy.datalog(),
                    inverter: healthy.serial(),
                    device_function: DeviceFunction::ReadInput,
                    register: 0,
                    values: vec![1; 254],
                },
            )))
            .unwrap();
        assert!(next_message(&mut mqtt)
            .await
            .topic
            .starts_with(&healthy.datalog().to_string()));
        coordinator.stop();
    };
    let (result, ()) = tokio::join!(coordinator.start(), assertions);
    result.unwrap();
}

#[tokio::test]
async fn failed_connection_without_coordinator_keeps_retrying_and_can_stop() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut entry = Factory::inverter();
    entry.host = "127.0.0.1".into();
    entry.port = listener.local_addr().unwrap().port();
    drop(listener); // Refuse the first TCP connection.
    let config = Factory::example_config_wrapped();
    config.set_inverters(vec![entry.clone()]);
    let channels = Channels::new();
    let inverter = Inverter::new(config, &entry, channels.clone());
    let assertions = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let status = channels.inverter_status.snapshot()[&entry.datalog()].clone();
        assert_eq!(status.state, lxp::status::ConnectionState::Disconnected);
        assert_eq!(status.connection_attempts, 1);
        assert!(status.last_error.is_some());
        inverter.stop();
    };
    timeout(Duration::from_millis(500), async {
        let (result, ()) = tokio::join!(inverter.start(), assertions);
        result.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn coordinator_supports_all_inverters_disabled() {
    let (config, _, _) = two_inverters();
    config.set_inverters(vec![]);
    let coordinator = Coordinator::new(config, Channels::new());
    let assertions = async {
        tokio::task::yield_now().await;
        coordinator.stop();
    };
    let (result, ()) = tokio::join!(coordinator.start(), assertions);
    result.unwrap();
}

#[tokio::test]
async fn commands_for_one_inverter_remain_ordered() {
    let (config, _, healthy) = two_inverters();
    let channels = Channels::new();
    let coordinator = Coordinator::new(config, channels.clone());
    let mut outgoing = channels.to_inverter.subscribe();
    let assertions = async {
        for register in [12, 13] {
            channels
                .from_mqtt
                .send(mqtt::ChannelData::Message(mqtt::Message {
                    topic: format!("cmd/{}/read/hold/{}", healthy.datalog(), register),
                    retain: false,
                    payload: String::new(),
                }))
                .unwrap();
        }
        let request = outgoing.recv().await.unwrap();
        assert_eq!(unwrap_inverter_channeldata_packet(request).register(), 12);
        assert!(timeout(Duration::from_millis(50), outgoing.recv())
            .await
            .is_err());
        channels
            .from_inverter
            .send(InverterData::Packet(holding(&healthy, 12)))
            .unwrap();
        let request = timeout(Duration::from_millis(500), outgoing.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(unwrap_inverter_channeldata_packet(request).register(), 13);
        coordinator.stop();
    };
    let (result, ()) = tokio::join!(coordinator.start(), assertions);
    result.unwrap();
}
