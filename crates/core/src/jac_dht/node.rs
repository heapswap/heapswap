use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};

use crate::{
	crypto::{
		address::Address,
		keys::{KeyPair, PublicKey},
	},
	traits::*,
};
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct Node {
	#[getset(get = "pub")]
	ipv4: Ipv4Addr,
	#[getset(get = "pub")]
	ipv4_port: u16,
	#[getset(get = "pub")]
	ipv6: Option<Ipv6Addr>,
	#[getset(get = "pub")]
	ipv6_port: Option<u16>,
}

impl Node {
	pub fn new(
		ipv4: Ipv4Addr,
		ipv4_port: u16,
		ipv6: Option<Ipv6Addr>,
		ipv6_port: Option<u16>,
	) -> Self {
		Node {
			ipv4,
			ipv4_port,
			ipv6,
			ipv6_port,
		}
	}
}

#[derive(Clone, Serialize, Deserialize, Getters, Setters)]
pub struct RemoteNode {
	#[getset(get = "pub")]
	node: Node,

	#[getset(get = "pub")]
	public_key: PublicKey,

	#[getset(get = "pub")]
	dist_to_self: f64,

	#[getset(get = "pub", set)]
	ping_ms: u32,
}

impl RemoteNode {
	pub fn new(
		node: Node,
		public_key: PublicKey,
		ping_ms: u32,
		local_node: LocalNode,
	) -> Self {
		let dist_to_self = local_node.dist_to_address(&public_key.address());
		RemoteNode {
			node,
			public_key,
			dist_to_self,
			ping_ms,
		}
	}
}

#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct LocalNode {
	#[getset(get = "pub")]
	pub node: Node,
	#[getset(get = "pub")]
	pub key_pair: KeyPair,
}

impl LocalNode {
	pub fn new(node: Node, key_pair: KeyPair) -> Self {
		LocalNode { node, key_pair }
	}

	pub fn dist_to_address(&self, address: &Address) -> f64 {
		self.key_pair().public_key().address().jaccard(&address)
	}
}
