use std::collections::{BTreeMap, HashMap};

use super::HashAlgorithm;

/// Represents a consistent hash ring with virtual nodes.
#[derive(Debug, Clone)]
pub struct HashRing {
	algorithm: HashAlgorithm,
	virtual_nodes: usize,
	ring: BTreeMap<u64, String>,
	counts: HashMap<String, usize>,
}

impl HashRing {
	pub fn new(virtual_nodes: usize, algorithm: HashAlgorithm) -> Self {
		Self {
			algorithm,
			virtual_nodes: virtual_nodes.max(1),
			ring: BTreeMap::new(),
			counts: HashMap::new(),
		}
	}

	pub fn add_server_with_weight(&mut self, server: &str, weight: usize) {
		if self.counts.contains_key(server) {
			self.remove_server(server);
		}
		let weight = weight.max(1);
		let replicas = self.virtual_nodes * weight;
		for idx in 0..replicas {
			let virtual_node_id = format!("{}-{}", server, idx);
			let hash = self.algorithm.hash(&virtual_node_id);
			self.ring.insert(hash, server.to_string());
		}
		self.counts.insert(server.to_string(), replicas);
	}

	pub fn remove_server(&mut self, server: &str) {
		if let Some(replicas) = self.counts.remove(server) {
			for idx in 0..replicas {
				let virtual_node_id = format!("{}-{}", server, idx);
				let hash = self.algorithm.hash(&virtual_node_id);
				self.ring.remove(&hash);
			}
		}
	}

	pub fn get_server(&self, key: &str) -> Option<&str> {
		if self.ring.is_empty() {
			return None;
		}
		let hash = self.algorithm.hash(key);
		let (_, server) = self
			.ring
			.range(hash..)
			.next()
			.or_else(|| self.ring.iter().next())?;
		Some(server.as_str())
	}

	pub fn nodes(&self) -> Vec<(u64, String)> {
		self.ring
			.iter()
			.map(|(hash, backend)| (*hash, backend.clone()))
			.collect()
	}
}
