use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::{
	crypto::keys::{KeyPair, PublicKey},
	traits::*,
};
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct Node {
	pub ipv4: Ipv4Addr,
	pub ipv4_port: u16,
	pub ipv6: Option<Ipv6Addr>,
	pub ipv6_port: Option<u16>,
}

#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct RemoteNode {
	pub node: Node,
	pub public_key: PublicKey,
	pub dist_to_self: u32,
	pub ping_ms: u32,
}

impl RemoteNode {
	pub fn new(
		node: Node,
		public_key: PublicKey,
		ping_ms: u32,
		self_node: &LocalNode,
	) -> Self {
		let dist_to_self = public_key
			.as_u256()
			.hamming(&self_node.key_pair.public_key().as_u256());
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
	pub node: Node,
	pub key_pair: KeyPair,
}

impl LocalNode {
	pub fn new(node: Node, key_pair: KeyPair) -> Self {
		LocalNode { node, key_pair }
	}
}
