mod common;
use common::*;
use lxp::status::{ConnectionState, InitializationState, StatusStore};

#[tokio::test]
async fn connection_lifecycle_errors_and_attempts_are_independent_of_registers() {
    let mut inverter = Factory::inverter();
    inverter.publish_holdings_on_connect = Some(true);
    let statuses = StatusStore::new();
    statuses.register(&inverter);
    let serial = inverter.datalog();
    assert_eq!(
        statuses.snapshot()[&serial].state,
        ConnectionState::Disconnected
    );
    statuses.connecting(serial);
    assert_eq!(
        statuses.snapshot()[&serial].state,
        ConnectionState::Connecting
    );
    statuses.authenticating(serial);
    assert_eq!(
        statuses.snapshot()[&serial].state,
        ConnectionState::Authenticating
    );
    statuses.disconnected(serial, Some("TLS handshake failed".into()));
    let failed = statuses.snapshot()[&serial].clone();
    assert_eq!(failed.state, ConnectionState::Disconnected);
    assert_eq!(failed.last_error.as_deref(), Some("TLS handshake failed"));
    assert!(failed.last_connected_at.is_none());
    assert!(failed.last_disconnected_at.is_some());
    statuses.connecting(serial);
    statuses.connected(serial);
    statuses.initialization(
        serial,
        2,
        InitializationState::Failed,
        Some("holding registers unavailable".into()),
    );
    let connected = statuses.snapshot()[&serial].clone();
    assert_eq!(connected.state, ConnectionState::Connected);
    assert_eq!(connected.connection_attempts, 2);
    assert!(connected.last_connected_at.is_some());
    assert!(connected.last_error.is_none());
    assert_eq!(connected.initialization, InitializationState::Failed);
    assert!(connected.initialization_error.is_some());
    assert!(connected.last_packet_received_at.is_none());
    statuses.disconnected(serial, None);
    statuses.connecting(serial);
    statuses.connected(serial);
    // Completion from a previous attempt cannot overwrite the new attempt.
    statuses.initialization(serial, 2, InitializationState::Ready, None);
    assert_eq!(
        statuses.snapshot()[&serial].initialization,
        InitializationState::Pending
    );
}

#[tokio::test]
async fn packet_updates_are_throttled_and_latest_state_survives_without_receivers() {
    let inverter = Factory::inverter();
    let statuses = StatusStore::new();
    statuses.register(&inverter);
    statuses.connecting(inverter.datalog());
    statuses.connected(inverter.datalog());
    let mut receiver = statuses.subscribe();
    receiver.borrow_and_update();
    statuses.packet_received(inverter.datalog(), false);
    assert!(receiver.has_changed().unwrap());
    receiver.borrow_and_update();
    statuses.packet_received(inverter.datalog(), false);
    assert!(!receiver.has_changed().unwrap());
    statuses.packet_received(inverter.datalog(), true);
    assert!(receiver.has_changed().unwrap());
    receiver.borrow_and_update();
    statuses.packet_received(inverter.datalog(), true);
    assert!(!receiver.has_changed().unwrap());
    assert!(statuses.snapshot()[&inverter.datalog()]
        .last_input_received_at
        .is_some());
    drop(receiver);
    // Independent of subscribers, MQTT delivery, and bounded telemetry queues.
    for _ in 0..3000 {
        statuses.connecting(inverter.datalog());
    }
    statuses.disconnected(inverter.datalog(), Some("connection refused".into()));
    let receiver = statuses.subscribe();
    assert_eq!(
        receiver.borrow()[&inverter.datalog()].state,
        ConnectionState::Disconnected
    );
    assert_eq!(
        receiver.borrow()[&inverter.datalog()].connection_attempts,
        3001
    );
}

#[tokio::test]
async fn coordinator_reports_register_failure_without_losing_connected_state() {
    let config = Factory::example_config_wrapped();
    let mut inverter = config.enabled_inverters()[0].clone();
    inverter.publish_holdings_on_connect = Some(true);
    config.set_inverters(vec![inverter.clone()]);
    let channels = Channels::new();
    channels.inverter_status.register(&inverter);
    channels.inverter_status.connecting(inverter.datalog());
    channels.inverter_status.connected(inverter.datalog());
    let mut status = channels.inverter_status.subscribe();
    let coordinator = Coordinator::new(config, channels.clone());
    let assertions = async {
        channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Connected(inverter.datalog()))
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            loop {
                if status.borrow_and_update()[&inverter.datalog()].initialization
                    == InitializationState::Failed
                {
                    break;
                }
                status.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let snapshot = channels.inverter_status.snapshot();
        assert_eq!(
            snapshot[&inverter.datalog()].state,
            ConnectionState::Connected
        );
        assert!(snapshot[&inverter.datalog()]
            .initialization_error
            .as_ref()
            .unwrap()
            .contains("to_inverter"));
        coordinator.stop();
    };
    let (result, ()) = tokio::join!(coordinator.start(), assertions);
    result.unwrap();
}
