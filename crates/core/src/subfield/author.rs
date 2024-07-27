use super::*;
use crate::*;
use getset::{Getters, Setters};

/**
 * RemoteAuthor
*/
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters)]
pub struct RemoteAuthor {
	#[getset(get = "pub")]
	public_key: crypto::PublicKey,
}

impl RemoteAuthor {
	pub fn new(public_key: crypto::PublicKey) -> Self {
		Self { public_key }
	}
}

/**
 * LocalAuthor
*/
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters)]
pub struct LocalAuthor {
	#[getset(get = "pub")]
	keypair: crypto::Keypair,
}

impl LocalAuthor {
	pub fn new(keypair: crypto::Keypair) -> Self {
		Self { keypair }
	}

	pub fn to_remote_author(&self) -> RemoteAuthor {
		RemoteAuthor::new(self.keypair.public_key().clone())
	}
}
