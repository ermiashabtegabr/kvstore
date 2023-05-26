mod grpc;
mod kvstore;

use grpc::connection::proto;
use serde::Deserialize;
use tide::Request;

#[derive(Clone, Debug, Deserialize)]
struct Get {
    key: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Set {
    key: String,
    value: u64,
}

fn _node_rpc_addr(id: usize) -> String {
    // Get the id of the node
    // Set the rpc address to <name>-<id>.<serviceName>
    format!("{}.serviceName", id)
}

#[async_std::main]
async fn main() -> tide::Result<()> {
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
    let _get_request_proto = proto::Get {
        key: get_request.key,
    };

    Ok(format!("huh").into())
}
