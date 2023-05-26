use super::kv::KeyValue;
use super::util::{ELECTION_TIMEOUT, OUTGOING_MESSAGE_PERIOD};
use crate::grpc::client::OmnipaxosTransport;
use omnipaxos::util::LogEntry;
use omnipaxos::{messages::Message, OmniPaxos, OmniPaxosConfig};
use omnipaxos_storage::memory_storage::MemoryStorage;
use std::sync::{Arc, Mutex};
use tokio::time;

type OmniPaxosKV<E> = OmniPaxos<E, MemoryStorage<E>>;

pub struct OmniPaxosServer<T: OmnipaxosTransport + Send + Sync> {
    omnipaxos: Arc<Mutex<OmniPaxosKV<KeyValue>>>,
    transport: Arc<T>,
    halt: Arc<Mutex<bool>>,
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
        let omnipaxos: Arc<Mutex<OmniPaxosKV<KeyValue>>> =
            Arc::new(Mutex::new(omnipaxos_config.build(MemoryStorage::default())));
        let halt = Arc::new(Mutex::new(false));

        Self {
            omnipaxos,
            transport: Arc::new(transport),
            halt,
        }
    }

    pub fn handle_set(&self, keyval: KeyValue) {
        self.omnipaxos
            .lock()
            .unwrap()
            .append(keyval)
            .expect("Failed to append");
    }

    pub fn handle_get(&self, key: String) -> Option<u64> {
        let committed_entries = self
            .omnipaxos
            .lock()
            .unwrap()
            .read_decided_suffix(0)
            .expect("Failed to read expected entries");

        for ent in committed_entries {
            if let LogEntry::Decided(kv) = ent {
                if kv.key == key {
                    return Some(kv.value);
                }
            }
            // ignore uncommitted entries
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
        self.omnipaxos.lock().unwrap().handle_incoming(message)
    }

    pub fn halt(&self, val: bool) {
        let mut halt = self.halt.lock().unwrap();
        *halt = val
    }
}
