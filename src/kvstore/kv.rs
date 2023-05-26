use omnipaxos::storage::{Entry, Snapshot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: u64,
}

impl Entry for KeyValue {
    type Snapshot = KVSnapshot;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KVSnapshot {
    snapshotted: HashMap<String, u64>,
}

impl KVSnapshot {
    pub fn with(snapshotted: HashMap<String, u64>) -> Self {
        KVSnapshot { snapshotted }
    }
    pub fn get_snapshot(&self) -> HashMap<String, u64> {
        self.snapshotted.clone()
    }
}

impl Snapshot<KeyValue> for KVSnapshot {
    fn create(entries: &[KeyValue]) -> Self {
        let mut snapshotted = HashMap::new();
        for e in entries {
            let KeyValue { key, value } = e;
            snapshotted.insert(key.clone(), *value);
        }
        Self { snapshotted }
    }

    fn merge(&mut self, delta: Self) {
        for (k, v) in delta.snapshotted {
            self.snapshotted.insert(k, v);
        }
    }

    fn use_snapshots() -> bool {
        true
    }
}
