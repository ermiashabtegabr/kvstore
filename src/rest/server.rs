use tide::http::convert::Deserialize;
use tide::prelude::*;
use tide::Request;

#[derive(Debug, Deserialize, Serialize)]
struct KeyValue {
    key: String,
    value: i32,
}

#[derive(Deserialize)]
struct Query {
    key: String,
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
    let key_val: KeyValue = match req.body_json().await {
        Ok(key_val) => key_val,
        Err(_) => KeyValue {
            key: String::from("nigga"),
            value: 10,
        },
    };

    Ok(format!("{} -> {}", key_val.key, key_val.value).into())
}

async fn get_value(req: Request<()>) -> tide::Result {
    let key = match req.query() {
        Ok(Query { key }) => key,
        Err(_) => "?key=key".into(),
    };

    Ok(format!("get -> {}", key).into())
}
