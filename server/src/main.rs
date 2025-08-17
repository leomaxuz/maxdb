use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::spawn;
use serde_json::{json, Value, Map};
mod protocol;
mod storage;
mod auth;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:65432").await.unwrap();
    println!("Server listening on 127.0.0.1:65432");

    let storage = Arc::new(tokio::sync::Mutex::new(storage::Storage::new()));
    let authdb = Arc::new(auth::AuthDB::new("config.json"));

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let storage = storage.clone();
        let authdb = authdb.clone();

        spawn(async move {
            loop {
                let req = match protocol::recv_msg(&mut socket).await {
                    Ok(v) => v,
                    Err(_) => break,
                };

                // &Value olish uchun
                let empty = Value::Object(Map::new());
                let auth = req.get("auth").unwrap_or(&empty);
                if !auth.is_object() || !authdb.verify(
                    auth.get("user").and_then(Value::as_str).unwrap_or(""),
                    auth.get("password").and_then(Value::as_str).unwrap_or("")
                ) {
                    protocol::send_msg(&mut socket, &json!({"status":"error","message":"auth_failed"})).await.unwrap();
                    continue;
                }

                let params = req.get("params").unwrap_or(&empty);
                let mut storage = storage.lock().await;

                match req.get("action").and_then(Value::as_str).unwrap_or("") {
                    "create_table" => {
                        let table = params["table"].as_str().unwrap_or_default();
                        let columns: Vec<String> = params["columns"].as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|v| v.as_str().unwrap_or("").to_string())
                            .collect();
                        let (ok, msg) = storage.create_table(table, columns);
                        protocol::send_msg(&mut socket, &json!({
                            "status": if ok {"ok"} else {"error"},
                            "message": msg
                        })).await.unwrap();
                    }
                    "insert" => {
                        let table = params["table"].as_str().unwrap_or_default();
                        let values: Vec<String> = params["values"].as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|v| v.as_str().unwrap_or("").to_string())
                            .collect();
                        let (ok, res) = storage.insert(table, values);
                        protocol::send_msg(&mut socket, &json!({
                            "status": if ok {"ok"} else {"error"},
                            "result": res
                        })).await.unwrap();
                    }
                    "select" => {
                        let table = params["table"].as_str().unwrap_or_default();
                        let (ok, rows) = storage.select(table);
                        protocol::send_msg(&mut socket, &json!({
                            "status": if ok {"ok"} else {"error"},
                            "rows": rows
                        })).await.unwrap();
                    }
                    _ => {
                        protocol::send_msg(&mut socket, &json!({
                            "status":"error",
                            "message":"unknown_action"
                        })).await.unwrap();
                    }
                }
            }
        });
    }
}
