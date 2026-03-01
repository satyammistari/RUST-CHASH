use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{config::{BackendConfig, ProxyConfig}, ring::HashRing};

pub mod client;
pub mod handler;

#[derive(Clone)]
pub struct AppState {
	pub ring: Arc<RwLock<HashRing>>,
	pub client: client::ProxyClient,
	pub proxy_config: ProxyConfig,
	pub backends: Arc<RwLock<Vec<BackendConfig>>>,
}

impl AppState {
	pub fn new(
		ring: HashRing,
		client: client::ProxyClient,
		proxy_config: ProxyConfig,
		backends: Vec<BackendConfig>,
	) -> Self {
		Self {
			ring: Arc::new(RwLock::new(ring)),
			client,
			proxy_config,
			backends: Arc::new(RwLock::new(backends)),
		}
	}

	pub async fn add_backend(&self, backend: BackendConfig) {
		let address = backend.address.clone();
		{
			let mut ring = self.ring.write().await;
			ring.add_server_with_weight(&address, backend.weight);
		}

		let mut backends = self.backends.write().await;
		if let Some(existing) = backends.iter_mut().find(|b| b.address == address) {
			*existing = backend;
		} else {
			backends.push(backend);
		}
	}

	pub async fn list_backends(&self) -> Vec<BackendConfig> {
		self.backends.read().await.clone()
	}
}
