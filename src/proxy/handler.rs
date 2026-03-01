use axum::{body::Body, extract::State, http::{Request, StatusCode}, response::Response};
use tracing::error;

use crate::{config::RoutingKeyStrategy, proxy::AppState};

pub async fn proxy_handler(
	State(state): State<AppState>,
	req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
	let key = derive_routing_key(&state, &req);
	let backend = {
		let ring = state.ring.read().await;
		ring.get_server(&key).map(str::to_string)
	}
	.ok_or(StatusCode::BAD_GATEWAY)?;

	state
		.client
		.forward(req, &backend)
		.await
		.map_err(|err| {
			error!(error = %err, "failed to proxy request");
			StatusCode::BAD_GATEWAY
		})
}

fn derive_routing_key(state: &AppState, req: &Request<Body>) -> String {
	match state.proxy_config.routing_key_strategy {
		RoutingKeyStrategy::Path => path_and_query(req),
		RoutingKeyStrategy::QueryParam => req
			.uri()
			.query()
			.map(|q| q.to_string())
			.filter(|q| !q.is_empty())
			.unwrap_or_else(|| path_and_query(req)),
		RoutingKeyStrategy::Header => {
			if let Some(header) = state.proxy_config.routing_header.as_deref() {
				if let Some(value) = req.headers().get(header) {
					if let Ok(v) = value.to_str() {
						if !v.is_empty() {
							return v.to_string();
						}
					}
				}
			}
			path_and_query(req)
		}
	}
}

fn path_and_query(req: &Request<Body>) -> String {
	req
		.uri()
		.path_and_query()
		.map(|pq| pq.as_str().to_string())
		.unwrap_or_else(|| "/".to_string())
}
