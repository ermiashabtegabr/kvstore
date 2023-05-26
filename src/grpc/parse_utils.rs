use super::connection::proto;
use crate::kvstore::kv::{KVSnapshot, KeyValue};

use omnipaxos::ballot_leader_election::Ballot;
use omnipaxos::messages::sequence_paxos::Compaction;
use omnipaxos::storage::{SnapshotType, StopSign};
use omnipaxos::util::SequenceNumber;

// ------------- struct -> proto -------------
pub fn get_proto_keyval(kv: KeyValue) -> proto::KeyValue {
    proto::KeyValue {
        key: kv.key,
        value: kv.value,
    }
}

pub fn get_proto_ballot(ballot: Ballot) -> Option<proto::Ballot> {
    Some(proto::Ballot {
        n: ballot.n,
        priority: ballot.priority,
        pid: ballot.pid,
    })
}

pub fn get_proto_snapshot_type(
    snapshot_type: SnapshotType<KeyValue>,
) -> Option<proto::SnapshotType> {
    match snapshot_type {
        SnapshotType::Delta(snapshot) => Some(proto::SnapshotType {
            snapshot: Some(proto::snapshot_type::Snapshot::Delta(proto::Snapshot {
                snapshot: snapshot.get_snapshot(),
            })),
        }),
        SnapshotType::Complete(snapshot) => Some(proto::SnapshotType {
            snapshot: Some(proto::snapshot_type::Snapshot::Complete(proto::Snapshot {
                snapshot: snapshot.get_snapshot(),
            })),
        }),
    }
}

pub fn get_proto_stopsign(stopsign: StopSign) -> Option<proto::StopSign> {
    let metadata = match stopsign.metadata {
        Some(meta) => meta.into_iter().map(|m| m as u32).collect(),
        _ => Vec::new(),
    };

    Some(proto::StopSign {
        config_id: stopsign.config_id,
        nodes: stopsign.nodes,
        metadata,
    })
}

pub fn get_proto_seq_num(seq_num: SequenceNumber) -> proto::SequenceNumber {
    proto::SequenceNumber {
        session: seq_num.session,
        counter: seq_num.counter,
    }
}

pub fn get_proto_compaction(compaction: Compaction) -> proto::compaction::Compaction {
    match compaction {
        Compaction::Trim(idx) => proto::compaction::Compaction::Trim(idx),
        Compaction::Snapshot(idx) => proto::compaction::Compaction::Snapshot(idx.unwrap()),
    }
}

// ------------- proto -> struct -------------
pub fn get_keyval_struct(keyval: proto::KeyValue) -> KeyValue {
    KeyValue {
        key: keyval.key,
        value: keyval.value,
    }
}

pub fn get_ballot_struct(ballot: proto::Ballot) -> Ballot {
    Ballot {
        n: ballot.n,
        priority: ballot.priority,
        pid: ballot.pid,
    }
}

pub fn get_stopsign_struct(stopsign: proto::StopSign) -> StopSign {
    let config_id = stopsign.config_id;
    let nodes = stopsign.nodes;
    let metadata = Some(stopsign.metadata.into_iter().map(|m| m as u8).collect());
    StopSign {
        config_id,
        nodes,
        metadata,
    }
}

pub fn get_seq_num_struct(seq_num: proto::SequenceNumber) -> SequenceNumber {
    SequenceNumber {
        session: seq_num.session,
        counter: seq_num.counter,
    }
}

pub fn get_snapshot_type_enum(snapshot_type: proto::SnapshotType) -> SnapshotType<KeyValue> {
    match snapshot_type.snapshot.unwrap() {
        proto::snapshot_type::Snapshot::Delta(snapshot) => {
            SnapshotType::Delta(KVSnapshot::with(snapshot.snapshot))
        }
        proto::snapshot_type::Snapshot::Complete(snapshot) => {
            SnapshotType::Complete(KVSnapshot::with(snapshot.snapshot))
        }
    }
}

pub fn get_compaction_enum(compaction: proto::compaction::Compaction) -> Compaction {
    match compaction {
        proto::compaction::Compaction::Trim(idx) => Compaction::Trim(idx),
        proto::compaction::Compaction::Snapshot(idx) => Compaction::Snapshot(Some(idx)),
    }
}
