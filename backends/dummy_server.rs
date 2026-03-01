use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use axum::{Router, extract::Request, routing::any, response::Json};
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "8081".to_string())
        .parse()
        .unwrap_or(8081);

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("Dummy backend #{port} listening on http://{addr}");

    let app = Router::new()
        .route("/*path", any(move |req: Request| async move {
            echo_handler(req, port).await
        }))
        .route("/", any(move |req: Request| async move {
            echo_handler(req, port).await
        }));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn echo_handler(req: Request, server_port: u16) -> Json<Value> {
    let method = req.method().to_string();
    let uri    = req.uri().to_string();

    let headers: serde_json::Map<String, Value> = req.headers()
        .iter()
        .map(|(name, value)| {
            let k = name.as_str().to_string();
            let v = value.to_str()
                .map(|s| Value::String(s.to_string()))
                .unwrap_or(Value::String("<binary>".to_string()));
            (k, v)
        })
        .collect();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Json(json!({
        "server": { "port": server_port, "id": format!("backend-{}", server_port) },
        "request": { "method": method, "uri": uri, "headers": headers },
        "message": format!("Hello from backend {}! Got {} {}", server_port, method, uri),
        "timestamp": timestamp,
    }))
}