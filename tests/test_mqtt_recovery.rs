mod common;
use common::*;

use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};

async fn read_frame(stream: &mut TcpStream) -> (u8, Vec<u8>) {
    let header = stream.read_u8().await.unwrap();
    let mut len = 0_usize;
    let mut shift = 0;
    loop {
        let byte = stream.read_u8().await.unwrap();
        len |= ((byte & 127) as usize) << shift;
        if byte & 128 == 0 {
            break;
        }
        shift += 7;
        assert!(shift <= 21);
    }
    let mut body = vec![0; len];
    stream.read_exact(&mut body).await.unwrap();
    (header, body)
}

async fn handshake(listener: &TcpListener) -> TcpStream {
    let (mut stream, _) = listener.accept().await.unwrap();
    assert_eq!(read_frame(&mut stream).await.0 >> 4, 1); // CONNECT
    stream.write_all(&[0x20, 2, 0, 0]).await.unwrap(); // CONNACK, fresh session
    stream
}

async fn verify_setup(stream: &mut TcpStream) {
    let mut online = false;
    let mut subscribed = false;
    while !online || !subscribed {
        let (header, body) = read_frame(stream).await;
        match header >> 4 {
            3 => {
                let length = u16::from_be_bytes([body[0], body[1]]) as usize;
                let topic = std::str::from_utf8(&body[2..2 + length]).unwrap();
                let packet_id = &body[2 + length..4 + length];
                if topic == "test/LWT" {
                    assert_eq!(&body[4 + length..], b"online");
                    online = true;
                }
                stream
                    .write_all(&[0x40, 2, packet_id[0], packet_id[1]])
                    .await
                    .unwrap();
            }
            8 => {
                let topic_len = u16::from_be_bytes([body[2], body[3]]) as usize;
                if &body[4..4 + topic_len] == b"test/cmd/all/#" {
                    subscribed = true;
                }
                stream
                    .write_all(&[0x90, 3, body[0], body[1], 0])
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn broker_reconnect_reannounces_online_and_resubscribes_and_ignores_bad_payloads() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let path = std::env::temp_dir().join(format!(
        "lxp-mqtt-recovery-{}-{}.json",
        std::process::id(),
        port
    ));
    std::fs::write(&path, json!({
        "inverters": [], "mqtt": {"host": "127.0.0.1", "port": port, "namespace": "test", "homeassistant": {"enabled": false}},
        "influx": {"enabled": false, "url": "http://localhost:8086", "database": "lxp"}
    }).to_string()).unwrap();
    let config = ConfigWrapper::new(path.to_str().unwrap().into()).unwrap();
    std::fs::remove_file(path).unwrap();
    let channels = Channels::new();
    let mqtt = Mqtt::new(config, channels.clone());
    let mut incoming = channels.from_mqtt.subscribe();
    let broker = async {
        let mut stream = handshake(&listener).await;
        verify_setup(&mut stream).await;
        drop(stream);
        // Exceed the bounded telemetry queue during the broker outage.
        for _ in 0..3000 {
            channels
                .to_mqtt
                .send(mqtt::ChannelData::Message(mqtt::Message {
                    topic: "sample".into(),
                    payload: "1".into(),
                    retain: false,
                }))
                .unwrap();
        }
        let mut stream = handshake(&listener).await;
        verify_setup(&mut stream).await;
        // UTF-8 decoding errors must not terminate the MQTT event loop.
        let topic = b"test/cmd/all/read/hold/12";
        for payload in [b"\xff".as_slice(), b"1".as_slice()] {
            let mut packet = vec![0x30, (2 + topic.len() + payload.len()) as u8];
            packet.extend_from_slice(&(topic.len() as u16).to_be_bytes());
            packet.extend_from_slice(topic);
            packet.extend_from_slice(payload);
            stream.write_all(&packet).await.unwrap();
        }
        assert_eq!(
            timeout(Duration::from_secs(1), incoming.recv())
                .await
                .unwrap()
                .unwrap(),
            mqtt::ChannelData::Message(mqtt::Message {
                topic: "cmd/all/read/hold/12".into(),
                retain: false,
                payload: "1".into()
            })
        );
    };
    timeout(Duration::from_secs(8), async {
        tokio::select! {
            result = mqtt.start() => panic!("MQTT stopped: {:?}", result),
            _ = broker => {},
        }
    })
    .await
    .unwrap();
}

async fn diagnostics_frame(stream: &mut TcpStream) -> (u8, String, Vec<u8>) {
    loop {
        let (header, body) = read_frame(stream).await;
        match header >> 4 {
            3 => {
                let length = u16::from_be_bytes([body[0], body[1]]) as usize;
                let topic = std::str::from_utf8(&body[2..2 + length])
                    .unwrap()
                    .to_string();
                assert_eq!((header >> 1) & 3, 1); // All bridge publications use QoS 1.
                let packet_id = &body[2 + length..4 + length];
                stream
                    .write_all(&[0x40, 2, packet_id[0], packet_id[1]])
                    .await
                    .unwrap();
                return (header, topic, body[4 + length..].to_vec());
            }
            8 => stream
                .write_all(&[0x90, 3, body[0], body[1], 0])
                .await
                .unwrap(),
            _ => {}
        }
    }
}

async fn read_status(stream: &mut TcpStream, state: &str) -> serde_json::Value {
    loop {
        let (header, topic, payload) = diagnostics_frame(stream).await;
        if topic == "test/2222222222/status" {
            assert_eq!(header & 1, 1, "status must be retained");
            let status: serde_json::Value = serde_json::from_slice(&payload).unwrap();
            if status["state"] == state {
                return status;
            }
        }
        assert!(!(topic == "test/LWT" && payload == b"offline"));
    }
}

async fn initial_statuses(stream: &mut TcpStream, active_state: &str) {
    let mut states = std::collections::HashMap::new();
    let mut discovery = std::collections::HashSet::new();
    loop {
        let (header, topic, payload) = diagnostics_frame(stream).await;
        if topic.ends_with("/connection_status/config") {
            assert_eq!(header & 1, 1);
            let config: serde_json::Value = serde_json::from_slice(&payload).unwrap();
            assert_eq!(config["availability"]["topic"], "test/LWT");
            discovery.insert(config["unique_id"].as_str().unwrap().to_string());
        }
        if topic == "test/2222222222/status" || topic == "test/3333333333/status" {
            assert_eq!(header & 1, 1);
            let status: serde_json::Value = serde_json::from_slice(&payload).unwrap();
            states.insert(topic.clone(), status["state"].as_str().unwrap().to_string());
        }
        if topic == "test/LWT" && payload == b"online" {
            // Correct retained states must precede bridge online, including on restart.
            assert_eq!(states.get("test/2222222222/status").unwrap(), active_state);
            assert_eq!(states.get("test/3333333333/status").unwrap(), "disabled");
            assert!(discovery.contains("lxp_2222222222_connection_status"));
            assert!(discovery.contains("lxp_3333333333_connection_status"));
            break;
        }
    }
}

#[tokio::test]
async fn status_is_available_without_inverter_data_and_replays_current_state_after_mqtt_reconnect()
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let path =
        std::env::temp_dir().join(format!("lxp-status-{}-{}.json", std::process::id(), port));
    std::fs::write(&path, json!({
        "inverters": [
            {"host": "127.0.0.1", "port": 1, "serial": "5555555555", "datalog": "2222222222", "publish_holdings_on_connect": true},
            {"enabled": false, "host": "127.0.0.1", "port": 2, "serial": "6666666666", "datalog": "3333333333"}
        ],
        "mqtt": {"host": "127.0.0.1", "port": port, "namespace": "test", "homeassistant": {"enabled": true}},
        "influx": {"enabled": false, "url": "http://localhost:8086", "database": "lxp"}
    }).to_string()).unwrap();
    let config = ConfigWrapper::new(path.to_str().unwrap().into()).unwrap();
    std::fs::remove_file(path).unwrap();
    let channels = Channels::new();
    let mqtt = Mqtt::new(config, channels.clone());
    let serial = "2222222222".parse().unwrap();
    // No inverter or coordinator tasks exist: discovery/status cannot depend on register reads.
    let broker = async {
        let mut stream = handshake(&listener).await;
        initial_statuses(&mut stream, "disconnected").await;
        let statuses = &channels.inverter_status;
        statuses.connecting(serial);
        read_status(&mut stream, "connecting").await;
        statuses.authenticating(serial);
        read_status(&mut stream, "authenticating").await;
        statuses.connected(serial);
        let connected = read_status(&mut stream, "connected").await;
        assert_eq!(connected["initialization"], "pending");
        assert!(connected["last_packet_received_at"].is_null());
        statuses.initialization(
            serial,
            1,
            lxp::status::InitializationState::Failed,
            Some("register read timed out".into()),
        );
        let failed = read_status(&mut stream, "connected").await;
        assert_eq!(failed["initialization"], "failed");
        assert_eq!(failed["initialization_error"], "register read timed out");
        statuses.disconnected(serial, Some("no data for 900 seconds".into()));
        let disconnected = read_status(&mut stream, "disconnected").await;
        assert_eq!(disconnected["last_error"], "no data for 900 seconds");
        drop(stream);
        // Several transitions during the outage must collapse to the latest status.
        statuses.connecting(serial);
        statuses.authenticating(serial);
        statuses.connected(serial);
        let mut stream = handshake(&listener).await;
        initial_statuses(&mut stream, "connected").await;
        // Even an unchanged state must be replayed after a fresh broker session.
        drop(stream);
        let mut stream = handshake(&listener).await;
        initial_statuses(&mut stream, "connected").await;
    };
    timeout(Duration::from_secs(16), async {
        tokio::select! {
            result = mqtt.start() => panic!("MQTT stopped: {:?}", result),
            _ = broker => {},
        }
    })
    .await
    .unwrap();
}
