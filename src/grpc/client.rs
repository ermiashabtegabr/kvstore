use async_trait::async_trait;
use derivative::Derivative;

use omnipaxos::messages::ballot_leader_election::{BLEMessage, HeartbeatMsg};
use omnipaxos::messages::sequence_paxos::{PaxosMessage, PaxosMsg};
use slog::{warn, Logger};

use super::connection::Connections;
use super::omnipaxos_grpc;
use super::parse_utils;
use crate::kvstore::kv::KeyValue;
use crate::logger;

#[async_trait]
pub trait OmnipaxosTransport {
    fn send_omnipaxos_message(&self, paxos_msg: PaxosMessage<KeyValue>);
    fn send_ble_message(&self, ble_msg: BLEMessage);
}

type NodeAddrFn = dyn Fn(usize) -> String + Send + Sync;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RpcTransport {
    /// Node address mapping function.
    #[derivative(Debug = "ignore")]
    node_addr: Box<NodeAddrFn>,
    connections: Connections,
    logger: Logger,
}

impl RpcTransport {
    pub fn new(node_addr: Box<NodeAddrFn>) -> Self {
        RpcTransport {
            node_addr,
            connections: Connections::new(),
            logger: logger::create_logger(),
        }
    }
}

#[async_trait]
impl OmnipaxosTransport for RpcTransport {
    fn send_omnipaxos_message(&self, paxos_msg: PaxosMessage<KeyValue>) {
        match paxos_msg.msg {
            PaxosMsg::PrepareReq => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let request = omnipaxos_grpc::PrepareReq { from, to };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }
                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    let response = client.conn.prepare_request(request).await;

                    match response {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::Prepare(prepare) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let n = parse_utils::get_proto_ballot(prepare.n);
                let decided_idx = prepare.decided_idx;
                let n_accepted = parse_utils::get_proto_ballot(prepare.n_accepted);
                let accepted_idx = prepare.accepted_idx;

                let request = omnipaxos_grpc::Prepare {
                    from,
                    to,
                    n,
                    decided_idx,
                    n_accepted,
                    accepted_idx,
                };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.prepare_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::Promise(promise) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let n = parse_utils::get_proto_ballot(promise.n);
                let n_accepted = parse_utils::get_proto_ballot(promise.n_accepted);
                let decided_snapshot = match promise.decided_snapshot {
                    Some(snapshot_type) => parse_utils::get_proto_snapshot_type(snapshot_type),
                    _ => None,
                };
                let suffix = promise
                    .suffix
                    .into_iter()
                    .map(parse_utils::get_proto_keyval)
                    .collect();
                let decided_idx = promise.decided_idx;
                let accepted_idx = promise.accepted_idx;
                let stopsign = match promise.stopsign {
                    Some(stopsign) => parse_utils::get_proto_stopsign(stopsign),
                    _ => None,
                };

                let request = omnipaxos_grpc::Promise {
                    from,
                    to,
                    n,
                    n_accepted,
                    decided_idx,
                    suffix,
                    decided_snapshot,
                    accepted_idx,
                    stopsign,
                };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.promise_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::AcceptSync(accept_sync) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let n = parse_utils::get_proto_ballot(accept_sync.n);
                let seq_num = parse_utils::get_proto_seq_num(accept_sync.seq_num);
                let decided_snapshot = match accept_sync.decided_snapshot {
                    Some(snapshot_type) => parse_utils::get_proto_snapshot_type(snapshot_type),
                    _ => None,
                };
                let suffix: Vec<omnipaxos_grpc::KeyValue> = accept_sync
                    .suffix
                    .into_iter()
                    .map(parse_utils::get_proto_keyval)
                    .collect();
                let sync_idx = accept_sync.sync_idx;
                let decided_idx = accept_sync.decided_idx;
                let stopsign = match accept_sync.stopsign {
                    Some(stopsign) => parse_utils::get_proto_stopsign(stopsign),
                    _ => None,
                };

                let request = omnipaxos_grpc::AcceptSync {
                    from,
                    to,
                    n,
                    seq_num: Some(seq_num),
                    decided_snapshot,
                    suffix,
                    sync_idx,
                    decided_idx,
                    stopsign,
                };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.accept_sync_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::AcceptDecide(accept_decide) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let n = parse_utils::get_proto_ballot(accept_decide.n);
                let seq_num = parse_utils::get_proto_seq_num(accept_decide.seq_num);
                let decided_idx = accept_decide.decided_idx;
                let entries: Vec<omnipaxos_grpc::KeyValue> = accept_decide
                    .entries
                    .into_iter()
                    .map(parse_utils::get_proto_keyval)
                    .collect();

                let request = omnipaxos_grpc::AcceptDecide {
                    from,
                    to,
                    seq_num: Some(seq_num),
                    n,
                    decided_idx,
                    entries,
                };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.accept_decide_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::Accepted(accepted) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let n = parse_utils::get_proto_ballot(accepted.n);
                let accepted_idx = accepted.accepted_idx;

                let request = omnipaxos_grpc::Accepted {
                    from,
                    to,
                    n,
                    accepted_idx,
                };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.accepted_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::Decide(decide) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let n = parse_utils::get_proto_ballot(decide.n);
                let seq_num = parse_utils::get_proto_seq_num(decide.seq_num);
                let decided_idx = decide.decided_idx;

                let request = omnipaxos_grpc::Decide {
                    from,
                    to,
                    n,
                    seq_num: Some(seq_num),
                    decided_idx,
                };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.decide_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::ProposalForward(proposals) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let proposals: Vec<omnipaxos_grpc::KeyValue> = proposals
                    .into_iter()
                    .map(parse_utils::get_proto_keyval)
                    .collect();

                let request = omnipaxos_grpc::ProposalForward {
                    from,
                    to,
                    proposals,
                };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.proposal_forward_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::Compaction(compaction) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let compaction = parse_utils::get_proto_compaction_type(compaction);

                let request = omnipaxos_grpc::Compaction {
                    from,
                    to,
                    compaction: Some(compaction),
                };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.compaction_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::AcceptStopSign(accept_ss) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let n = parse_utils::get_proto_ballot(accept_ss.n);
                // let seq_num = parse_utils::get_proto_seq_num(accept_ss.seq_num);
                let ss = parse_utils::get_proto_stopsign(accept_ss.ss);

                let request = omnipaxos_grpc::AcceptStopSign {
                    from,
                    to,
                    n,
                    // seq_num,
                    ss,
                };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.accept_stop_sign_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::AcceptedStopSign(accepted_ss) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let n = parse_utils::get_proto_ballot(accepted_ss.n);

                let request = omnipaxos_grpc::AcceptedStopSign { from, to, n };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.accepted_stop_sign_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::DecideStopSign(decide_ss) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let n = parse_utils::get_proto_ballot(decide_ss.n);
                // let seq_num = parse_utils ::get_proto_seq_num(decide_ss.seq_num);

                let request = omnipaxos_grpc::DecideStopSign { from, to, n };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.decide_stop_sign_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            PaxosMsg::ForwardStopSign(stopsign) => {
                let from = paxos_msg.from;
                let to = paxos_msg.to;

                let ss = parse_utils::get_proto_stopsign(stopsign);

                let request = omnipaxos_grpc::ForwardStopSign { from, to, ss };

                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to connect to peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.forward_stop_sign_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
        }
    }

    fn send_ble_message(&self, ble_msg: BLEMessage) {
        match ble_msg.msg {
            HeartbeatMsg::Request(heartbeat_request) => {
                let from = ble_msg.from;
                let to = ble_msg.to;

                let round = heartbeat_request.round;

                let request = omnipaxos_grpc::HeartbeatRequest { from, to, round };
                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.heartbeat_request_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
            HeartbeatMsg::Reply(heartbeat_reply) => {
                let from = ble_msg.from;
                let to = ble_msg.to;

                let round = heartbeat_reply.round;
                let ballot = parse_utils::get_proto_ballot(heartbeat_reply.ballot);
                let quorum_connected = heartbeat_reply.quorum_connected;

                let request = omnipaxos_grpc::HeartbeatReply {
                    from,
                    to,
                    round,
                    ballot,
                    quorum_connected,
                };
                let peer = (self.node_addr)(to as usize);
                let pool = self.connections.clone();
                let logger = self.logger.clone();
                tokio::task::spawn(async move {
                    while let Err(_) = pool.connection(&peer).await {
                        warn!(logger, "Failed to connect to peer {}", to);
                    }

                    let mut client = pool
                        .connection(&peer)
                        .await
                        .map_err(|_| warn!(logger, "Failed to find peer {}", to))
                        .unwrap();
                    let request = tonic::Request::new(request);
                    match client.conn.heartbeat_reply_message(request).await {
                        Ok(_) => {}
                        Err(_) => warn!(logger, "Peer {} halted", to),
                    }
                });
            }
        }
    }
}
