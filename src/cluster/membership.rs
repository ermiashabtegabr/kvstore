#![allow(dead_code)]
use regex::Regex;
use std::collections::HashMap;
use trust_dns_resolver::error::ResolveError;

use super::peer_discovery;

#[derive(Debug)]
pub struct NodeInfo {
    id: u64,
    api_port: String,
    grpc_port: String,
}

#[derive(Debug)]
pub struct Cluster {
    replica_map: HashMap<u32, String>,
}

fn get_node_id(node_name: String) -> u32 {
    let re = Regex::new(r"kvstore-(\d{1})").expect("Failed to create regex expression");
    let caps = re
        .captures(&node_name)
        .expect("Failed to capture node name");
    caps[1].parse::<u32>().expect("Failed to parse node id") + 1
}

impl Cluster {
    pub fn new() -> Self {
        Cluster {
            replica_map: HashMap::new(),
        }
    }

    pub async fn from_peer_discovery(dns_name: String) -> Result<Self, ResolveError> {
        let peers = peer_discovery::find_peers(dns_name).await?;
        let mut replica_map = HashMap::new();

        peers.into_iter().for_each(|node_name| {
            let node_id = get_node_id(node_name.to_string());
            replica_map.insert(node_id, node_name.to_string());
        });

        let cluster = Cluster { replica_map };
        Ok(cluster)
    }
}
