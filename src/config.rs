use std::{fmt, fs, path::Path};

use serde::{Deserialize, Serialize};

/// Top-level configuration for the proxy application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub proxy: ProxyConfig,
	#[serde(default)]
	pub backends: Vec<BackendConfig>,
}

impl Config {
	/// Load configuration from a TOML file path.
	pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
		let raw = fs::read_to_string(path).map_err(ConfigError::Io)?;
		toml::from_str(&raw).map_err(ConfigError::Toml)
	}
}

/// Runtime configuration for the proxy server itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
	#[serde(default = "default_listen_addr")]
	pub listen_addr: String,
	#[serde(default = "default_virtual_nodes")]
	pub virtual_nodes: usize,
	#[serde(default = "default_hash_algorithm")]
	pub hash_algorithm: String,
	#[serde(default)]
	pub routing_key_strategy: RoutingKeyStrategy,
	#[serde(default)]
	pub routing_header: Option<String>,
}

impl Default for ProxyConfig {
	fn default() -> Self {
		Self {
			listen_addr: default_listen_addr(),
			virtual_nodes: default_virtual_nodes(),
			hash_algorithm: default_hash_algorithm(),
			routing_key_strategy: RoutingKeyStrategy::default(),
			routing_header: None,
		}
	}
}

fn default_listen_addr() -> String {
	"0.0.0.0:8080".to_string()
}

fn default_virtual_nodes() -> usize {
	150
}

fn default_hash_algorithm() -> String {
	"fnv1a".to_string()
}

/// Configuration for each backend server in the ring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
	pub address: String,
	#[serde(default = "default_backend_weight")]
	pub weight: usize,
}

fn default_backend_weight() -> usize {
	1
}

/// Strategy used to derive the routing key that is hashed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingKeyStrategy {
	Path,
	Header,
	QueryParam,
}

impl Default for RoutingKeyStrategy {
	fn default() -> Self {
		RoutingKeyStrategy::Path
	}
}

/// Errors that can occur while loading configuration.
#[derive(Debug)]
pub enum ConfigError {
	Io(std::io::Error),
	Toml(toml::de::Error),
}

impl fmt::Display for ConfigError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ConfigError::Io(e) => write!(f, "I/O error: {}", e),
			ConfigError::Toml(e) => write!(f, "TOML parse error: {}", e),
		}
	}
}

impl std::error::Error for ConfigError {}

