mod grpc;
mod kvstore;
mod logger;

use anyhow::Result;
use grpc::client::RpcTransport;
use grpc::omnipaxos_grpc::{self, omni_paxos_protocol_server::OmniPaxosProtocolServer};
use grpc::server::OmniPaxosProtocolService;
use kvstore::server::OmniPaxosServer;
use serde::Deserialize;
use std::sync::Arc;
use structopt::StructOpt;
use tide::Request;
use tonic::transport::Server;

#[derive(StructOpt, Debug)]
#[structopt(name = "kvstore")]
struct Opt {
    /// The ID of this server.
    #[structopt(short, long)]
    id: usize,
    /// The IDs of peers.
    #[structopt(short, long, required = false)]
    peers: Vec<usize>,
}

#[derive(Clone, Debug, Deserialize)]
struct Get {
    key: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Set {
    key: String,
    value: u64,
}

fn node_authority(id: usize) -> (&'static str, u16) {
    let host = "127.0.0.1";
    let port = 50000 + (id as u16);
    (host, port)
}

fn node_rpc_addr(id: usize) -> String {
    // Get the id of the node
    // Set the rpc address to <name>-<id>.<serviceName>
    let (host, port) = node_authority(id);
    format!("http://{}:{}", host, port)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut opt = Opt::from_args();
    let peers = opt.peers.iter_mut().map(|peer| *peer as u64).collect();
    let (host, port) = node_authority(opt.id);
    let rpc_listen_addr = format!("{}:{}", host, port).parse().unwrap();
    let transport = RpcTransport::new(Box::new(node_rpc_addr));
    let server = OmniPaxosServer::new(opt.id as u64, peers, transport);
    let server = Arc::new(server);

    let http_server = tokio::task::spawn(async move {
        if opt.id != 1 {
            return;
        }
        start_http_server()
            .await
            .expect("Failed to start http server");
    });

    let event_loop = {
        let server = server.clone();
        tokio::task::spawn(async move {
            server.start_message_event_loop().await;
        })
    };

    let rpc = OmniPaxosProtocolService::new(server);
    let grpc_server = tokio::task::spawn(async move {
        println!("RPC listening to {} ...", rpc_listen_addr);
        let ret = Server::builder()
            .add_service(OmniPaxosProtocolServer::new(rpc))
            .serve(rpc_listen_addr)
            .await;
        ret
    });

    let _results = tokio::try_join!(http_server, grpc_server, event_loop)?;

    Ok(())
}

async fn start_http_server() -> tide::Result<()> {
    println!("[INFO] listening on 0.0.0.0:8080");

    let mut server = tide::new();
    server.at("/").get(|_| async { Ok("PING") });
    server.at("/set").post(set_value);
    server.at("/get").get(get_value);
    server.listen("0.0.0.0:8080").await?;

    Ok(())
}

async fn set_value(mut req: Request<()>) -> tide::Result {
    let Set { key, value } = req.body_json().await?;
    println!("{}:{}", key, value);
    Ok(format!("{} -> {}", key, value).into())
}

async fn get_value(req: Request<()>) -> tide::Result {
    let get_request: Get = req.query().expect("Failed to parde query");
    let _get_request_proto = omnipaxos_grpc::Get {
        key: get_request.key,
    };

    Ok("huh".to_string().into())
}
