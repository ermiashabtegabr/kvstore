mod grpc;
mod kvstore;
mod logger;

use anyhow::Result;
use grpc::client::{OmnipaxosTransport, RpcTransport};
use grpc::omnipaxos_grpc::omni_paxos_protocol_server::OmniPaxosProtocolServer;
use grpc::server::OmniPaxosProtocolService;
use kvstore::kv::KeyValue;
use kvstore::server::OmniPaxosServer;
use serde::Deserialize;
use std::sync::Arc;
use structopt::StructOpt;
use tide::Request;
use tonic::transport::Server;

#[derive(StructOpt, Debug)]
#[structopt(name = "kvstore")]
struct Opt {
    #[structopt(short, long)]
    id: usize,
    #[structopt(short, long, required = false)]
    peers: Vec<usize>,
}

#[derive(Clone, Debug, Deserialize)]
struct Key {
    pub key: String,
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
    let omnipaxos_server = OmniPaxosServer::new(opt.id as u64, peers, transport);
    let omnipaxos_server = Arc::new(omnipaxos_server);

    if opt.id == 1 {
        let omnipaxos_server = omnipaxos_server.clone();
        tokio::task::spawn(async move {
            femme::start();
            let mut http_server = tide::with_state(omnipaxos_server);
            http_server.with(tide::log::LogMiddleware::new());
            http_server.at("/").get(|_| async { Ok("PING") });
            http_server.at("/set").post(set_value);
            http_server.at("/get").get(get_value);
            http_server
                .listen("127.0.0.1:8080")
                .await
                .expect("Failed to set listener");
        });
    }

    let event_loop = {
        let omnipaxos_server = omnipaxos_server.clone();
        tokio::task::spawn(async move {
            omnipaxos_server.start_message_event_loop().await;
        })
    };

    let rpc = OmniPaxosProtocolService::new(omnipaxos_server);
    let grpc_server = tokio::task::spawn(async move {
        println!("RPC listening to {} ...", rpc_listen_addr);
        let ret = Server::builder()
            .add_service(OmniPaxosProtocolServer::new(rpc))
            .serve(rpc_listen_addr)
            .await;
        ret
    });

    let _results = tokio::try_join!(grpc_server, event_loop)?;

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
    let value = omnipaxos_server.handle_get(&key.key).unwrap();

    Ok(format!("[{}] -> [{}]", key.key, value))
}
