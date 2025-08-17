use tokio::net::TcpStream;
use serde_json::{json, Value};
use serde::Deserialize;
use std::fs;
use std::env;
mod protocol;

#[derive(Deserialize)]
struct Config {
    host: String,
    user: String,
    password: String,
}

fn read_config(path: &str) -> Config {
    let data = fs::read_to_string(path).expect("Cannot read config file");
    serde_json::from_str(&data).expect("Invalid config format")
}

async fn send_request(host: &str, user: &str, password: &str, action: &str, params: Value) -> Value {
    let mut stream = TcpStream::connect(host).await.unwrap();

    let req: Value = json!({
        "action": action,
        "params": params,
        "auth": {"user": user, "password": password}
    });

    protocol::send_msg(&mut stream, &req).await.unwrap();
    protocol::recv_msg(&mut stream).await.unwrap()
}

#[tokio::main]
async fn main() {
    let cfg = read_config("config.json");  // Configni o‘qiymiz
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: client.exe <action> [params in JSON]");
        return;
    }

    let action = &args[1];
    let params: Value = if args.len() >= 3 {
        serde_json::from_str(&args[2]).unwrap_or(json!({}))
    } else {
        json!({})
    };

    let resp = send_request(&cfg.host, &cfg.user, &cfg.password, action, params).await;
    println!("{}", resp);
}
