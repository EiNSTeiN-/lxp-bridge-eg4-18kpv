//! Connection diagnostics live independently of register polling and telemetry queues.
use crate::prelude::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::watch;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Disabled,
    Disconnected,
    Connecting,
    Authenticating,
    Connected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InitializationState {
    NotRequested,
    Pending,
    Reading,
    Ready,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InverterStatus {
    pub state: ConnectionState,
    pub state_since: String,
    pub connection_attempts: u64,
    pub last_error: Option<String>,
    pub last_connected_at: Option<String>,
    pub last_disconnected_at: Option<String>,
    pub last_packet_received_at: Option<String>,
    pub last_input_received_at: Option<String>,
    pub initialization: InitializationState,
    pub initialization_error: Option<String>,
    #[serde(skip)]
    packet_reported_at: Option<Instant>,
}

impl InverterStatus {
    fn new(inverter: &config::Inverter) -> Self {
        Self {
            state: if inverter.enabled() {
                ConnectionState::Disconnected
            } else {
                ConnectionState::Disabled
            },
            state_since: now(),
            connection_attempts: 0,
            last_error: None,
            last_connected_at: None,
            last_disconnected_at: None,
            last_packet_received_at: None,
            last_input_received_at: None,
            initialization: if inverter.enabled() && inverter.publish_holdings_on_connect() {
                InitializationState::Pending
            } else {
                InitializationState::NotRequested
            },
            initialization_error: None,
            packet_reported_at: None,
        }
    }

    fn transition(&mut self, state: ConnectionState) {
        if self.state != state {
            self.state = state;
            self.state_since = now();
        }
    }
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub type Snapshot = HashMap<Serial, InverterStatus>;

#[derive(Clone, Debug)]
pub struct StatusStore(Rc<watch::Sender<Snapshot>>);

impl StatusStore {
    pub fn new() -> Self {
        Self(Rc::new(watch::channel(HashMap::new()).0))
    }

    pub fn subscribe(&self) -> watch::Receiver<Snapshot> {
        self.0.subscribe()
    }

    pub fn snapshot(&self) -> Snapshot {
        self.0.borrow().clone()
    }

    pub fn register(&self, inverter: &config::Inverter) {
        self.0.send_if_modified(|statuses| {
            if statuses.contains_key(&inverter.datalog()) {
                return false;
            }
            statuses.insert(inverter.datalog(), InverterStatus::new(inverter));
            true
        });
    }

    fn update(&self, datalog: Serial, update: impl FnOnce(&mut InverterStatus)) {
        self.0.send_if_modified(|statuses| {
            if let Some(status) = statuses.get_mut(&datalog) {
                update(status);
                true
            } else {
                false
            }
        });
    }

    pub fn connecting(&self, datalog: Serial) {
        self.update(datalog, |status| {
            status.transition(ConnectionState::Connecting);
            status.connection_attempts += 1;
            status.packet_reported_at = None;
            if status.initialization != InitializationState::NotRequested {
                status.initialization = InitializationState::Pending;
            }
            status.initialization_error = None;
        });
    }

    pub fn authenticating(&self, datalog: Serial) {
        self.update(datalog, |status| {
            status.transition(ConnectionState::Authenticating)
        });
    }

    pub fn connected(&self, datalog: Serial) {
        self.update(datalog, |status| {
            status.transition(ConnectionState::Connected);
            status.last_connected_at = Some(now());
            status.last_error = None;
        });
    }

    pub fn disconnected(&self, datalog: Serial, error: Option<String>) {
        self.update(datalog, |status| {
            status.transition(ConnectionState::Disconnected);
            status.last_disconnected_at = Some(now());
            if error.is_some() {
                status.last_error = error;
            }
            if status.initialization != InitializationState::NotRequested {
                status.initialization = InitializationState::Pending;
            }
        });
    }

    pub fn packet_received(&self, datalog: Serial, is_input: bool) {
        self.0.send_if_modified(|statuses| {
            let Some(status) = statuses.get_mut(&datalog) else {
                return false;
            };
            status.last_packet_received_at = Some(now());
            let first_input = is_input && status.last_input_received_at.is_none();
            if is_input {
                status.last_input_received_at = status.last_packet_received_at.clone();
            }
            // Keep exact times in memory; report packet freshness at most once a minute
            // (plus the first input) so HA does not record a new status for every packet.
            if first_input
                || status
                    .packet_reported_at
                    .map_or(true, |time| time.elapsed() >= Duration::from_secs(60))
            {
                status.packet_reported_at = Some(Instant::now());
                true
            } else {
                false
            }
        });
    }

    pub fn initialization(
        &self,
        datalog: Serial,
        attempt: u64,
        state: InitializationState,
        error: Option<String>,
    ) {
        self.0.send_if_modified(|statuses| {
            let Some(status) = statuses.get_mut(&datalog) else {
                return false;
            };
            // A cancelled initializer from an earlier connection must not overwrite
            // the new connection's diagnostics, or turn a disconnected sensor online.
            if status.state != ConnectionState::Connected
                || status.connection_attempts != attempt
                || status.initialization == InitializationState::NotRequested
            {
                return false;
            }
            status.initialization = state;
            status.initialization_error = error;
            true
        });
    }
}

impl Default for StatusStore {
    fn default() -> Self {
        Self::new()
    }
}
