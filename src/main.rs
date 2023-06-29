#![allow(dead_code, unused_imports, unused_variables)]
mod cluster;
mod error;
mod grpc;
mod kvstore;
mod logger;

use anyhow::Result;
use grpc::client::{OmnipaxosTransport, RpcTransport};
use grpc::omnipaxos_grpc::omni_paxos_protocol_server::OmniPaxosProtocolServer;
use grpc::server::OmniPaxosProtocolService;
use kvstore::{kv::KeyValue, server::OmniPaxosServer};
use regex::Regex;
use serde::Deserialize;
use std::sync::Arc;
use structopt::StructOpt;
use tide::Request;
use tonic::transport::Server;

#[derive(StructOpt, Debug)]
#[structopt(name = "kvstore")]
struct Opt {
    #[structopt(short, long)]
    node: String,
    #[structopt(short, long)]
    replicas: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct Key {
    pub key: String,
}

fn get_node_id(node_name: String) -> u32 {
    let re = Regex::new(r"kvstore-(\d{1})").expect("Failed to create regex expression");
    let caps = re
        .captures(&node_name)
        .expect("Failed to capture node name");
    caps[1].parse::<u32>().expect("Failed to parse node id") + 1
}

fn node_authority() -> (&'static str, u16) {
    let host = "kvstore-hs.kvstore-k8s.svc.cluster.local";
    let port = 5000;
    (host, port)
}

fn node_rpc_addr(id: usize) -> String {
    let (host, port) = node_authority();
    format!("http://kvstore-{}.{}:{}", id - 1, host, port)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    let node_id = get_node_id(opt.node);
    let peers = (1..opt.replicas + 1)
        .collect::<Vec<usize>>()
        .iter()
        .map(|i| *i as u64)
        .filter(|i| *i != node_id as u64)
        .collect();
    let rpc_listen_addr = format!("0.0.0.0:5000")
        .parse()
        .expect("Failed to parse socket address");

    println!("**** lol starting node {node_id} - peers {peers:?} ****");
    let transport = RpcTransport::new(Box::new(node_rpc_addr));
    let omnipaxos_server = OmniPaxosServer::new(node_id as u64, peers, transport);
    let omnipaxos_server = Arc::new(omnipaxos_server);

    let client_server = {
        let omnipaxos_server = omnipaxos_server.clone();
        tokio::task::spawn(async move {
            femme::start();
            let mut http_server = tide::with_state(omnipaxos_server);
            http_server.with(tide::log::LogMiddleware::new());
            http_server.at("/").get(|_| async { Ok("Hello") });
            http_server
                .at("/healthz")
                .get(|_| async { Ok("Health check!") });
            http_server.at("/set").post(set_value);
            http_server.at("/get").get(get_value);
            http_server
                .listen("0.0.0.0:8080")
                .await
                .expect("Failed to set listener");
        })
    };

    let event_loop = {
        let omnipaxos_server = omnipaxos_server.clone();
        tokio::task::spawn(async move {
            omnipaxos_server.start_message_event_loop().await;
        })
    };

    let grpc = OmniPaxosProtocolService::new(omnipaxos_server);
    let grpc_server = tokio::task::spawn(async move {
        let ret = Server::builder()
            .add_service(OmniPaxosProtocolServer::new(grpc))
            .serve(rpc_listen_addr)
            .await;
        ret
    });

    let _results = tokio::try_join!(client_server)?;

    // let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    //
    // let peer_discovery = runtime.spawn(async {
    //     cluster::peer_discovery::find_peers(String::new()).await;
    // });
    //
    // std::thread::sleep(std::time::Duration::from_secs(1000));

    Ok(())
}

type State<T> = Arc<OmniPaxosServer<T>>;

async fn set_value<T>(mut req: Request<State<T>>) -> tide::Result
where
    T: OmnipaxosTransport + Send + Sync,
{
    let keyval: KeyValue = req.body_json().await.expect("Failed to parse request body");
    let omnipaxos_server = req.state();
    omnipaxos_server.handle_set(keyval);
    Ok(tide::Response::new(200))
}

async fn get_value<T>(req: Request<State<T>>) -> tide::Result<String>
where
    T: OmnipaxosTransport + Send + Sync,
{
    let key: Key = req.query()?;
    let omnipaxos_server = req.state();
    let value = omnipaxos_server.handle_get(&key.key);

    Ok(format!("[{}] -> [{:?}]", key.key, value))
}
