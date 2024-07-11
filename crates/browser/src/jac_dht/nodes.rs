use crate::crypto::keys;
use getset::{Getters, Setters};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

// This represents a node in the network
// likely has networking information
type Node = JsValue;

// This is a local node
#[wasm_bindgen]
#[derive(Clone, Getters)]
pub struct LocalNode {
	node: Node,
	keypair: keys::Keypair,
}

#[wasm_bindgen]
impl LocalNode {
	/**
	 * Constructors
		*/
	#[wasm_bindgen(constructor)]
	pub fn new(node: &Node, keypair: &keys::Keypair) -> LocalNode {
		LocalNode {
			node: node.clone(),
			keypair: keypair.clone(),
		}
	}

	/**
	 * Getters
		*/
	#[wasm_bindgen(getter)]
	pub fn node(&self) -> Node {
		self.node.clone()
	}

	#[wasm_bindgen(getter)]
	pub fn keypair(&self) -> keys::Keypair {
		self.keypair.clone()
	}
}

// This is a remote node
#[wasm_bindgen]
#[derive(Clone, Getters, Setters)]
pub struct RemoteNode {
	node: Node,
	public_key: keys::PublicKey,
	dist_to_local: f64,
	ping_ms: u32,
}

#[wasm_bindgen]
impl RemoteNode {
	/**
	 * Constructors
		*/
	#[wasm_bindgen(constructor)]
	pub fn new(
		remote_node: &Node,
		public_key: &keys::PublicKey,
		local_node: &LocalNode,
		ping_ms: u32,
	) -> RemoteNode {
		RemoteNode {
			node: remote_node.clone(),
			public_key: public_key.clone(),
			dist_to_local: local_node
				.keypair()
				.public_key()
				.jaccard(&public_key),
			ping_ms: ping_ms,
		}
	}

	// for testing
	#[wasm_bindgen]
	pub fn random(local_node: &LocalNode, remote_node: &Node) -> RemoteNode {
		let keypair = keys::Keypair::random();
		let ping_ms = OsRng.next_u32() % 100;
		RemoteNode::new(remote_node, &keypair.public_key(), local_node, ping_ms)
	}

	/**
	 * Getters
		*/
	#[wasm_bindgen(getter)]
	pub fn node(&self) -> Node {
		self.node.clone()
	}

	#[wasm_bindgen(getter, js_name = "publicKey")]
	pub fn public_key(&self) -> keys::PublicKey {
		self.public_key.clone()
	}

	#[wasm_bindgen(getter, js_name = "distToLocal")]
	pub fn dist_to_local(&self) -> f64 {
		self.dist_to_local
	}

	#[wasm_bindgen(getter, js_name = "pingMs")]
	pub fn ping_ms(&self) -> u32 {
		self.ping_ms
	}

	/**
	 * Setters
		*/
	#[wasm_bindgen(setter, js_name = "pingMs")]
	pub fn set_ping_ms(&mut self, ping_ms: u32) {
		self.ping_ms = ping_ms;
	}
}
