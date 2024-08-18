use super::*;
use crate::*;

#[derive(Debug)]
pub enum NodeError {
	InvalidSwarmConfig,
}

#[derive(Clone, Getters)]
pub struct RemoteNode {
	#[getset(get = "pub")]
	public_key: crypto::PublicKey,
	#[getset(get = "pub", set = "pub")]
	ping_ms: u64,
}

impl RemoteNode {
	pub fn new(public_key: crypto::PublicKey, ping_ms: u64) -> Self {
		Self {
			public_key,
			ping_ms,
		}
	}
}

#[derive(Clone, Getters)]
pub struct LocalNode {
	#[getset(get = "pub")]
	public_key: crypto::PublicKey,
	#[getset(get = "pub")]
	private_key: crypto::PrivateKey,
}

impl LocalNode {
	pub fn new(private_key: crypto::PrivateKey) -> Self {
		let public_key = private_key.public_key();
		Self {
			public_key,
			private_key,
		}
	}

	pub fn to_remote_node(&self) -> RemoteNode {
		RemoteNode {
			public_key: self.public_key.clone(),
			ping_ms: 0,
		}
	}
}
