use super::omnipaxos_grpc as grpc;
use crate::kvstore::kv::{KVSnapshot, KeyValue};

use omnipaxos::ballot_leader_election::Ballot;
use omnipaxos::messages::sequence_paxos::Compaction;
use omnipaxos::storage::{SnapshotType, StopSign};
use omnipaxos::util::SequenceNumber;

// ------------- struct -> proto -------------
pub fn get_proto_keyval(kv: KeyValue) -> grpc::KeyValue {
    grpc::KeyValue {
        key: kv.key,
        value: kv.value,
    }
}

pub fn get_proto_ballot(ballot: Ballot) -> Option<grpc::Ballot> {
    Some(grpc::Ballot {
        n: ballot.n,
        priority: ballot.priority,
        pid: ballot.pid,
    })
}

pub fn get_proto_snapshot_type(
    snapshot_type: SnapshotType<KeyValue>,
) -> Option<grpc::SnapshotType> {
    match snapshot_type {
        SnapshotType::Delta(snapshot) => Some(grpc::SnapshotType {
            snapshot: Some(grpc::snapshot_type::Snapshot::Delta(grpc::Snapshot {
                snapshot: snapshot.get_snapshot(),
            })),
        }),
        SnapshotType::Complete(snapshot) => Some(grpc::SnapshotType {
            snapshot: Some(grpc::snapshot_type::Snapshot::Complete(grpc::Snapshot {
                snapshot: snapshot.get_snapshot(),
            })),
        }),
    }
}

pub fn get_proto_stopsign(stopsign: StopSign) -> Option<grpc::StopSign> {
    let metadata = match stopsign.metadata {
        Some(meta) => meta.into_iter().map(|m| m as u32).collect(),
        _ => Vec::new(),
    };

    Some(grpc::StopSign {
        config_id: stopsign.config_id,
        nodes: stopsign.nodes,
        metadata,
    })
}

pub fn get_proto_seq_num(seq_num: SequenceNumber) -> grpc::SequenceNumber {
    grpc::SequenceNumber {
        session: seq_num.session,
        counter: seq_num.counter,
    }
}

pub fn get_proto_compaction_type(compaction: Compaction) -> grpc::compaction::Compaction {
    match compaction {
        Compaction::Trim(idx) => grpc::compaction::Compaction::Trim(idx),
        Compaction::Snapshot(idx) => grpc::compaction::Compaction::Snapshot(idx.unwrap()),
    }
}

// ------------- proto -> struct -------------
pub fn get_keyval_struct(keyval: grpc::KeyValue) -> KeyValue {
    KeyValue {
        key: keyval.key,
        value: keyval.value,
    }
}

pub fn get_ballot_struct(ballot: grpc::Ballot) -> Ballot {
    Ballot {
        n: ballot.n,
        priority: ballot.priority,
        pid: ballot.pid,
    }
}

pub fn get_stopsign_struct(stopsign: grpc::StopSign) -> StopSign {
    let config_id = stopsign.config_id;
    let nodes = stopsign.nodes;
    let metadata = Some(stopsign.metadata.into_iter().map(|m| m as u8).collect());
    StopSign {
        config_id,
        nodes,
        metadata,
    }
}

pub fn get_seq_num_struct(seq_num: grpc::SequenceNumber) -> SequenceNumber {
    SequenceNumber {
        session: seq_num.session,
        counter: seq_num.counter,
    }
}

pub fn get_snapshot_type_enum(snapshot_type: grpc::SnapshotType) -> SnapshotType<KeyValue> {
    match snapshot_type.snapshot.unwrap() {
        grpc::snapshot_type::Snapshot::Delta(snapshot) => {
            SnapshotType::Delta(KVSnapshot::with(snapshot.snapshot))
        }
        grpc::snapshot_type::Snapshot::Complete(snapshot) => {
            SnapshotType::Complete(KVSnapshot::with(snapshot.snapshot))
        }
    }
}

pub fn get_compaction_enum(compaction: grpc::compaction::Compaction) -> Compaction {
    match compaction {
        grpc::compaction::Compaction::Trim(idx) => Compaction::Trim(idx),
        grpc::compaction::Compaction::Snapshot(idx) => Compaction::Snapshot(Some(idx)),
    }
}
