pub mod omnipaxos_grpc {
    tonic::include_proto!("omnipaxos");
}

pub mod client;
pub mod connection;
pub mod parse_utils;
pub mod server;
