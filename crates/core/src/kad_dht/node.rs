use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};

use crate::{
	crypto::keys::{KeyPair, PublicKey},
	traits::*,
};
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct Node {
	#[getset(get = "pub")]
	pub ipv4: Ipv4Addr,
	#[getset(get = "pub")]
	pub ipv4_port: u16,
	#[getset(get = "pub")]
	pub ipv6: Option<Ipv6Addr>,
	#[getset(get = "pub")]
	pub ipv6_port: Option<u16>,
}

#[derive(Clone, Serialize, Deserialize, Getters, Setters)]
pub struct RemoteNode {
	#[getset(get = "pub")]
	pub node: Node,
	#[getset(get = "pub")]
	pub public_key: PublicKey,
	#[getset(get = "pub", set)]
	pub ping_ms: u32,
}

impl RemoteNode {
	pub fn new(node: Node, public_key: PublicKey, ping_ms: u32) -> Self {
		RemoteNode {
			node,
			public_key,
			ping_ms,
		}
	}
}

#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct LocalNode {
	pub node: Node,
	pub key_pair: KeyPair,
}

impl LocalNode {
	pub fn new(node: Node, key_pair: KeyPair) -> Self {
		LocalNode { node, key_pair }
	}
}
