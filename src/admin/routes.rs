use std::collections::HashMap;

use axum::{extract::State, http::StatusCode, response::{Html, IntoResponse, Json}, routing::get, Router};
use serde::{Deserialize, Serialize};

use crate::{config::{BackendConfig, ProxyConfig}, proxy::AppState};

use super::visualizer::visualizer_page;

pub fn admin_router() -> Router<AppState> {
	Router::<AppState>::new()
		.route("/ring", get(ring_distribution))
		.route("/ring/visualize", get(ring_visualization))
		.route("/config", get(config_snapshot))
		.route("/visualizer", get(visualizer_handler))
		.route("/servers", get(list_servers).post(add_server))
}

#[derive(Serialize)]
struct RingEntry {
	hash: u64,
	backend: String,
}

async fn ring_distribution(State(state): State<AppState>) -> Json<Vec<RingEntry>> {
	let ring = state.ring.read().await;
	let payload = ring
		.nodes()
		.into_iter()
		.map(|(hash, backend)| RingEntry { hash, backend })
		.collect();
	Json(payload)
}

#[derive(Serialize)]
struct BackendVisualization {
	backend: String,
	nodes: usize,
}

#[derive(Serialize)]
struct RingVisualization {
	total_nodes: usize,
	backends: Vec<BackendVisualization>,
}

async fn ring_visualization(State(state): State<AppState>) -> Json<RingVisualization> {
	let ring = state.ring.read().await;
	let nodes = ring.nodes();
	let mut distribution: HashMap<String, usize> = HashMap::new();
	for (_, backend) in &nodes {
		*distribution.entry(backend.clone()).or_default() += 1;
	}

	let backends = distribution
		.into_iter()
		.map(|(backend, nodes)| BackendVisualization { backend, nodes })
		.collect();

	Json(RingVisualization {
		total_nodes: nodes.len(),
		backends,
	})
}

async fn config_snapshot(State(state): State<AppState>) -> Json<ProxyConfig> {
	Json(state.proxy_config.clone())
}

#[derive(Serialize)]
struct ServersResponse {
	servers: Vec<BackendConfig>,
}

async fn list_servers(State(state): State<AppState>) -> Json<ServersResponse> {
	let servers = state.list_backends().await;
	Json(ServersResponse { servers })
}

#[derive(Deserialize)]
struct AddServerPayload {
	address: String,
	#[serde(default = "default_weight")]
	weight: usize,
}

const fn default_weight() -> usize {
	1
}

async fn add_server(
	State(state): State<AppState>,
	Json(payload): Json<AddServerPayload>,
) -> Result<Json<BackendConfig>, (StatusCode, String)> {
	if payload.address.trim().is_empty() {
		return Err((StatusCode::BAD_REQUEST, "address cannot be empty".into()));
	}

	let backend = BackendConfig {
		address: payload.address,
		weight: payload.weight.max(1),
	};

	state.add_backend(backend.clone()).await;
	Ok(Json(backend))
}

async fn visualizer_handler(State(state): State<AppState>) -> impl IntoResponse {
	let ring = state.ring.read().await;
	Html(visualizer_page(&ring))
}
