use super::kv::KeyValue;
use super::util::{ELECTION_TIMEOUT, OUTGOING_MESSAGE_PERIOD};
use crate::grpc::client::OmnipaxosTransport;
use crate::logger;

use commitlog::LogOptions;
use omnipaxos::util::LogEntry;
use omnipaxos::{messages::Message, OmniPaxos, OmniPaxosConfig};
use omnipaxos_storage::persistent_storage::{PersistentStorage, PersistentStorageConfig};
use sled::Config;
use slog::{info, Logger};
use std::sync::{Arc, Mutex};
use tokio::time;

type OmniPaxosKV<E> = OmniPaxos<E, PersistentStorage<E>>;

pub struct OmniPaxosServer<T: OmnipaxosTransport + Send + Sync> {
    pid: u64,
    omnipaxos: Arc<Mutex<OmniPaxosKV<KeyValue>>>,
    transport: Arc<T>,
    logger: Logger,
}

impl<T: OmnipaxosTransport + Send + Sync> OmniPaxosServer<T> {
    pub fn new(pid: u64, peers: Vec<u64>, transport: T) -> Self {
        let configuration_id = 1;
        let omnipaxos_config = OmniPaxosConfig {
            configuration_id,
            pid,
            peers,
            ..Default::default()
        };

        let my_path = format!("/node/storage/node-{}", pid);
        let my_log_opts = LogOptions::new(my_path.clone());
        let my_sled_opts = Config::default().path(my_path.clone());

        let mut my_config = PersistentStorageConfig::default();
        my_config.set_path(my_path);
        my_config.set_commitlog_options(my_log_opts);
        my_config.set_database_options(my_sled_opts);

        let omnipaxos: Arc<Mutex<OmniPaxosKV<KeyValue>>> = Arc::new(Mutex::new(
            omnipaxos_config.build(PersistentStorage::open(my_config)),
        ));
        let logger = logger::create_logger();

        Self {
            pid,
            omnipaxos,
            transport: Arc::new(transport),
            logger,
        }
    }

    pub fn handle_set(&self, keyval: KeyValue) {
        info!(self.logger, "Replica {} received set request", self.pid);
        self.omnipaxos
            .lock()
            .unwrap()
            .append(keyval)
            .expect("Failed to append");
    }

    pub fn handle_get(&self, key: &String) -> Option<u64> {
        info!(self.logger, "Replica {} received get request", self.pid);
        let committed_entries = self
            .omnipaxos
            .lock()
            .unwrap()
            .read_decided_suffix(0)
            .expect("Failed to read expected entries");

        for ent in committed_entries {
            if let LogEntry::Decided(kv) = ent {
                if kv.key == *key {
                    return Some(kv.value);
                }
            }
        }

        Some(0)
    }

    pub async fn send_outgoing_msgs(&self) {
        let messages = self.omnipaxos.lock().unwrap().outgoing_messages();
        for msg in messages {
            match msg {
                Message::SequencePaxos(paxos_msg) => {
                    self.transport.send_omnipaxos_message(paxos_msg)
                }
                Message::BLE(ble_msg) => self.transport.send_ble_message(ble_msg),
            }
        }
    }

    pub async fn start_message_event_loop(&self) {
        info!(self.logger, "Replica {} starting event loop", self.pid);
        let mut outgoing_interval = time::interval(OUTGOING_MESSAGE_PERIOD);
        let mut election_interval = time::interval(ELECTION_TIMEOUT);
        loop {
            tokio::select! {
                biased;
                _ = election_interval.tick() => { self.omnipaxos.lock().unwrap().election_timeout(); },
                _ = outgoing_interval.tick() => { self.send_outgoing_msgs().await; },
                else => { }
            }
        }
    }

    pub fn receive_message(&self, message: Message<KeyValue>) {
        info!(self.logger, "Replica {} received message", self.pid);
        self.omnipaxos.lock().unwrap().handle_incoming(message)
    }
}
