use std::hash::Hasher;

/// Hash algorithms supported by the ring.
#[derive(Debug, Clone, Copy)]
pub enum HashAlgorithm {
	Fnv1a,
	SipHash,
}

impl HashAlgorithm {
	pub fn from_str(input: &str) -> Self {
		match input.to_ascii_lowercase().as_str() {
			"siphash" | "sip" => HashAlgorithm::SipHash,
			_ => HashAlgorithm::Fnv1a,
		}
	}

	pub fn hash(&self, key: &str) -> u64 {
		match self {
			HashAlgorithm::Fnv1a => fnv1a_hash(key.as_bytes()),
			HashAlgorithm::SipHash => sip_hash(key.as_bytes()),
		}
	}
}

impl Default for HashAlgorithm {
	fn default() -> Self {
		HashAlgorithm::Fnv1a
	}
}

fn fnv1a_hash(bytes: &[u8]) -> u64 {
	const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
	const FNV_PRIME: u64 = 0x100000001b3;

	bytes.iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
		(hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
	})
}

fn sip_hash(bytes: &[u8]) -> u64 {
	let mut hasher = std::collections::hash_map::DefaultHasher::new();
	hasher.write(bytes);
	hasher.finish()
}
