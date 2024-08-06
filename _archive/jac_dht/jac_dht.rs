use super::nodes::*;
use crate::arr::{hamming, xor};
use crate::constants::NS;
use crate::crypto::keys;
use crate::misc::traits::*;
use crate::u256;
use crate::{crypto::keys::KeyArr, u256::*};
use getset::{CopyGetters, Getters, Setters};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

const K: u32 = 32;

#[wasm_bindgen]
#[derive(Debug)]
pub enum JacDHTError {
	InvalidKeyArr,
	InvalidKeyPair,
}

#[wasm_bindgen]
#[derive(Clone, Getters)]
pub struct JacDHT {
	#[getset(get = "pub")]
	max_dist_nodes: u32,
	#[getset(get = "pub")]
	max_ping_nodes: u32,

	#[getset(get = "pub")]
	local_node: LocalNode,

	// the remote_node in this is the source of truth and should be used to update the other two
	#[getset(get = "pub")]
	remote_nodes: HashMap<KeyArr, RemoteNode>,

	#[getset(get = "pub")]
	remote_nodes_by_dist: PriorityQueue<KeyArr, OrderedFloat<f64>>,
	#[getset(get = "pub")]
	remote_nodes_by_ping: PriorityQueue<KeyArr, u32>,
}

type EvictedNode = RemoteNode;

#[wasm_bindgen]
pub struct NearestNode {
	#[wasm_bindgen(skip)]
	pub nearest_node: RemoteNode,
	#[wasm_bindgen(skip)]
	pub dist: f64,
}

#[wasm_bindgen]
impl NearestNode {
	#[wasm_bindgen(getter, js_name = node)]
	pub fn nearest_node(&self) -> RemoteNode {
		self.nearest_node.clone()
	}

	#[wasm_bindgen(getter)]
	pub fn dist(&self) -> f64 {
		self.dist
	}
}

#[wasm_bindgen]
impl JacDHT {
	#[wasm_bindgen(constructor)]
	pub fn new(
		local_node: LocalNode,
		max_dist_nodes: u32,
		max_ping_nodes: u32,
	) -> JacDHT {
		JacDHT {
			max_dist_nodes,
			max_ping_nodes,
			local_node,
			remote_nodes: HashMap::new(),
			remote_nodes_by_dist: PriorityQueue::new(),
			remote_nodes_by_ping: PriorityQueue::new(),
		}
	}

	/*
	 Add Node
	*/

	fn add_remote_node_by_dist(&mut self, remote_node: RemoteNode) {
		let key_arr = remote_node.public_key().u256().data_u8().clone();

		// Add to remote_nodes_by_dist
		self.remote_nodes_by_dist
			.push(key_arr.clone(), OrderedFloat(remote_node.dist_to_local()));

		// Add to remote_nodes
		self.remote_nodes.insert(key_arr, remote_node);
	}

	fn add_remote_node_by_ping(&mut self, remote_node: RemoteNode) {
		let key_arr = remote_node.public_key().u256().data_u8().clone();

		// Add to remote_nodes_by_dist
		self.remote_nodes_by_ping
			.push(key_arr.clone(), remote_node.ping_ms());

		// Add to remote_nodes
		self.remote_nodes.insert(key_arr, remote_node);
	}

	/**
	 * Update Node
		*/
	fn update_remote_node_by_dist(&mut self, remote_node: RemoteNode) {
		let key_arr = remote_node.public_key().u256().data_u8().clone();

		// Modify remote_nodes_by_dist
		self.remote_nodes_by_dist.change_priority(
			&key_arr,
			OrderedFloat(remote_node.dist_to_local()),
		);

		// Modify remote_nodes
		self.remote_nodes.insert(key_arr, remote_node);
	}

	fn update_remote_node_by_ping(&mut self, remote_node: RemoteNode) {
		let key_arr = remote_node.public_key().u256().data_u8().clone();

		// Modify remote_nodes_by_dist
		self.remote_nodes_by_ping
			.change_priority(&key_arr, remote_node.ping_ms());

		// Modify remote_nodes
		self.remote_nodes.insert(key_arr, remote_node);
	}

	/**
	 * Remove Node
		*/
	fn remove_remote_node_by_dist(
		&mut self,
		key_arr: &KeyArr,
	) -> Option<EvictedNode> {
		// Remove from remote_nodes_by_dist
		self.remote_nodes_by_dist.remove(key_arr);

		// Remove from remote_nodes
		let evicted = self.remote_nodes.remove(key_arr);

		evicted
	}

	fn remove_remote_node_by_ping(
		&mut self,
		key_arr: &KeyArr,
	) -> Option<EvictedNode> {
		// Remove from remote_nodes_by_dist
		self.remote_nodes_by_ping.remove(key_arr);

		// Remove from remote_nodes
		let evicted = self.remote_nodes.remove(key_arr);

		evicted
	}

	/**
	 * Try Add Node
		*/
	#[wasm_bindgen(js_name = tryAddNode)]
	pub fn try_add_node(
		&mut self,
		remote_node: RemoteNode,
	) -> Option<EvictedNode> {
		//    self.try_evict(remote_node.clone(), true)
		//        .or_else(|| self.try_evict(remote_node, false))
		//}

		//fn try_evict(&mut self, remote_node: RemoteNode, is_testing_dist: bool) -> Option<EvictedNode> {
		//let (queue, max_queue_len) = if is_testing_dist {
		//    (&mut self.remote_nodes_by_dist, self.max_dist_nodes)
		//} else {
		//    (&mut self.remote_nodes_by_ping, self.max_ping_nodes)
		//};

		let key_arr = remote_node.public_key().u256().data_u8().clone();

		// Return None if the node is already in the JacDHT
		if self.remote_nodes.contains_key(&key_arr) {
			return None;
		}

		/*
		// Determine if eviction is necessary
		let should_evict = queue.len() as u32 >= max_queue_len
			&& queue.peek().map_or(false, |(_, &farthest_dist)| {
				remote_node.dist_to_local() < farthest_dist
			});

		// Evict the farthest node if necessary
		let evicted = if should_evict {
			queue.peek().map(|(key, _)| key.clone()).and_then(|key| {

				if is_testing_dist {
					let evicted = self.remove_remote_node_by_dist(&key);

					// try to add the evicted node, to see if it can be added for ping
					if let Some(evicted) = evicted {
						return self.try_add_node(evicted)
					}

					evicted

				} else {
					self.remove_remote_node_by_ping(&key)
				}
			})
		} else {
			None
		};

		// Return None if the node is already in the JacDHT
		if self.remote_nodes.contains_key(&key_arr) {
			return None;
		}

		// Add the new node if it's not replacing an existing one
		if !should_evict || evicted.is_some() {
			if is_testing_dist {
				self.add_remote_node_by_dist(remote_node);
			} else {
				self.add_remote_node_by_ping(remote_node);
			}
		}

		evicted
		*/

		// First, check distance

		// check if distance is full
		if self.remote_nodes_by_dist.len() as u32 >= self.max_dist_nodes {
			// check if the new node is closer than the farthest node
			if let Some((&farthest_key, &farthest_dist)) =
				self.remote_nodes_by_dist.peek()
			{
				if OrderedFloat(remote_node.dist_to_local()) < farthest_dist {
					// remove the farthest node
					let evicted =
						self.remove_remote_node_by_dist(&farthest_key);

					// add the new node
					self.add_remote_node_by_dist(remote_node.clone());

					// try to add evicted to ping
					if let Some(evicted) = evicted {
						return self.try_add_node(evicted);
					} else {
						return None;
					}
				}
			}
		} else {
			// If the distance queue is not full, add the node
			self.add_remote_node_by_dist(remote_node.clone());
			return None;
		}

		// Second, check ping

		// check if ping is full
		if self.remote_nodes_by_ping.len() as u32 >= self.max_ping_nodes {
			// check if the new node is closer than the farthest node
			if let Some((&farthest_key, &farthest_ping)) =
				self.remote_nodes_by_ping.peek()
			{
				if remote_node.ping_ms() < farthest_ping {
					// remove the farthest node
					let evicted =
						self.remove_remote_node_by_ping(&farthest_key);

					// add the new node
					self.add_remote_node_by_ping(remote_node);

					return evicted;
				}
			}
		} else {
			// If the ping queue is not full, add the node
			self.add_remote_node_by_ping(remote_node);
			return None;
		}

		None
	}

	#[wasm_bindgen(js_name = tryRemoveNode)]
	pub fn try_remove_node(
		&mut self,
		remote_node: &RemoteNode,
	) -> Option<RemoteNode> {
		let key_arr = remote_node.public_key().u256().data_u8().clone();

		let removed = self.remote_nodes.remove(&key_arr);
		if removed.is_some() {
			self.remote_nodes_by_dist.remove(&key_arr);
			self.remote_nodes_by_ping.remove(&key_arr);
		}
		removed
	}

	/**
	 * Nearest Nodes
		*/
	#[wasm_bindgen(js_name = nearestNodesToLocalByDist)]
	pub fn nearest_nodes_to_local_by_dist(&self, n: usize) -> Vec<RemoteNode> {
		let mut sorted_nodes: Vec<_> = self
			.remote_nodes_by_dist
			.iter()
			.map(|(key, &dist)| {
				(self.remote_nodes.get(key).unwrap().clone(), dist)
			})
			.collect();
		sorted_nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
		sorted_nodes
			.into_iter()
			.map(|(node, _)| node)
			.take(n)
			.collect()
	}

	#[wasm_bindgen(js_name = nearestNodesToLocalByPing)]
	pub fn nearest_nodes_to_local_by_ping(&self, n: usize) -> Vec<RemoteNode> {
		let mut sorted_nodes: Vec<_> = self
			.remote_nodes_by_ping
			.iter()
			.map(|(key, &ping)| {
				(self.remote_nodes.get(key).unwrap().clone(), ping)
			})
			.collect();
		sorted_nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
		sorted_nodes
			.into_iter()
			.map(|(node, _)| node)
			.take(n)
			.collect()
	}

	/**
	 * Absolute Nearest Node by dist to a KeyArr
		*/
	#[wasm_bindgen(js_name = nearestNode)]
	pub fn nearest_node_by_dist(
		&self,
		key_address: &U256,
	) -> Option<NearestNode> {
		let nearest_node =
			self.remote_nodes.iter().min_by_key(|(_, remote_node)| {
				OrderedFloat(
					remote_node.public_key().u256().jaccard(key_address),
				)
			});

		match nearest_node {
			//Some(key) => {
			Some((_, nearest_node)) => {
				let dist =
					nearest_node.public_key().u256().jaccard(key_address);
				Some(NearestNode {
					nearest_node: nearest_node.clone(),
					dist,
				})
			}
			None => None,
		}
	}

	#[wasm_bindgen(js_name = nearestNodes)]
	pub fn nearest_n_nodes_by_dist(
		&self,
		key_address: &U256,
		n: usize,
	) -> Vec<NearestNode> {
		let mut sorted_nodes: Vec<_> = self
			.remote_nodes_by_dist
			.iter()
			.map(|(key, &dist)| {
				let dist = self
					.remote_nodes
					.get(key)
					.unwrap()
					.public_key()
					.u256()
					.jaccard(key_address);
				(self.remote_nodes.get(key).unwrap().clone(), dist)
			})
			.collect();
		sorted_nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
		sorted_nodes
			.into_iter()
			.take(n)
			.map(|(node, dist)| NearestNode {
				nearest_node: node,
				dist,
			})
			.collect()
	}
}

/*
#[test]
fn test_jac_dht() -> Result<(), Box<dyn std::error::Error>> {
	let dummy_node: Node =
		Node::new(Ipv4Addr::new(127, 0, 0, 1), 1234, None, None);

	const DUMMY_NODE_COUNT: usize = 128;
	let mut dummy_nodes = Vec::with_capacity(DUMMY_NODE_COUNT);

	let local_node = LocalNode::new(dummy_node.clone(), KeyPair::random());

	// create remote nodes
	for _ in 0..DUMMY_NODE_COUNT {
		let key_pair = KeyPair::random();
		let remote_node = RemoteNode::new(
			dummy_node.clone(),
			key_pair.public_key().clone(),
			0,
			local_node.clone(),
		);
		dummy_nodes.push(remote_node);
	}

	let mut dht = JacDHT::new(local_node.clone());

	// add the dummy nodes
	for node in dummy_nodes.iter() {
		dht.try_add_node(node.clone());
	}

	const TRIALS: usize = 10;

	for _ in 0..TRIALS {
		// find nearest nodes to a random key
		let key = Address::random();

		// print the key distance
		println!(
			"{:.5} - key distance to self",
			dht.local_node().dist_to_address(&key)
		);

		let nearest_nodes = dht.nearest_node_by_dist(&key);

		// print the nearest node's public key
		if let Some(nearest_node) = nearest_nodes {
			println!(
				"{:.5} - {}",
				nearest_node.dist,
				nearest_node.nearest_node.public_key().to_string()
			);
		}
	}

	let key = Address::random();

	// time the nearest node calculation
	let s = timeit::timeit_loops!(1000, {
		let nearest_node = dht.nearest_node_by_dist(&key);
		core::hint::black_box(&nearest_node);
	});

	println!("Nearest Node: {:?}ns/loop", s * NS as f64);

	println!(
		"Node counts {}/{}",
		dht.remote_nodes_by_dist.len(),
		dht.remote_nodes_by_ping.len()
	);

	Ok(())
}

*/
