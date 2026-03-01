mod config;
mod ring;
mod proxy;
mod admin;

use std::net::SocketAddr;
use axum::{Router, routing::get};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use ring::{HashRing, HashAlgorithm};
use proxy::{AppState, client::ProxyClient};
use proxy::handler::proxy_handler;
use admin::admin_router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "consistent_hash_proxy=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Consistent Hash Proxy...");

    let config = Config::load("config.toml").unwrap_or_else(|e| {
        tracing::warn!("Could not load config.toml ({}), using defaults", e);
        Config {
            proxy: config::ProxyConfig::default(),
            backends: vec![
                config::BackendConfig { address: "http://127.0.0.1:8081".into(), weight: 1 },
                config::BackendConfig { address: "http://127.0.0.1:8082".into(), weight: 1 },
                config::BackendConfig { address: "http://127.0.0.1:8083".into(), weight: 1 },
            ],
        }
    });

    let algorithm = HashAlgorithm::from_str(&config.proxy.hash_algorithm);
    let mut ring  = HashRing::new(config.proxy.virtual_nodes, algorithm);

    for backend in &config.backends {
        ring.add_server_with_weight(&backend.address, backend.weight);
        info!("Added backend: {}", backend.address);
    }

    let client = ProxyClient::new();
    let state  = AppState::new(ring, client, config.proxy.clone(), config.backends.clone());

    let app = Router::new()
        .nest("/admin", admin_router())
        .route("/healthz", get(|| async { "OK" }))
        .fallback(proxy_handler)
        .with_state(state);

    let listen_addr: SocketAddr = config.proxy.listen_addr.parse()?;
    info!("Proxy listening on http://{}", listen_addr);

    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    info!("Shutting down...");
}
