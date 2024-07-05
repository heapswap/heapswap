use crate::crypto::keys::KeyPair;
use crate::{bys::*, misc::*, traits::*};
use dashmap::DashMap;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, sync::Arc};

use super::node::*;

type EvictedNode = Arc<RemoteNode>;

const ALPHA: usize = 3; // number of nodes to ping
const K: usize = 20; // max number of nodes in the buckets
const D: usize = 256; // number of bits in the key

#[derive(Clone, Getters)]
pub struct Bucket {
	#[getset(get = "pub")]
	k: usize, // max number of nodes in the bucket
	#[getset(get = "pub")]
	remote_nodes: Vec<Arc<RemoteNode>>,
}

pub trait DHTStore: Send + Sync {
	fn get(&self, key: U256) -> Option<Bytes>;
	fn set(&self, key: U256, value: Bytes);
}

#[derive(Clone)]
pub struct MemoryStore {
	store: DashMap<U256, Bytes>,
}

impl MemoryStore {
	pub fn new() -> Self {
		MemoryStore {
			store: DashMap::new(),
		}
	}
}

impl DHTStore for MemoryStore {
	fn get(&self, key: U256) -> Option<Bytes> {
		self.store.get(&key).map(|v| v.clone())
	}

	fn set(&self, key: U256, value: Bytes) {
		self.store.insert(key, value);
	}
}

#[derive(Getters)]
pub struct KadDHT {
	#[getset(get = "pub")]
	local_node: Arc<LocalNode>,
	#[getset(get = "pub")]
	remote_nodes: DashMap<U256, Arc<RemoteNode>>,
	#[getset(get = "pub")]
	buckets: Vec<Bucket>,
	#[getset(get = "pub", get_mut = "pub")]
	store: Box<dyn DHTStore + 'static>,
}

#[derive(Clone, Getters)]
pub struct NearRemoteNode {
	#[getset(get = "pub")]
	distance: usize,
	#[getset(get = "pub")]
	remote_node: Arc<RemoteNode>,
}

impl KadDHT {
	pub fn new(local_node: LocalNode) -> Self {
		let mut buckets = Vec::with_capacity(D);
		for _ in 0..D {
			buckets.push(Bucket {
				k: K,
				remote_nodes: Vec::with_capacity(K),
			});
		}

		// add self to the DHT
		let self_remote_node = RemoteNode::new(
			Node {
				ipv4: Ipv4Addr::new(127, 0, 0, 1),
				ipv4_port: 1234,
				ipv6: None,
				ipv6_port: None,
			},
			local_node.key_pair.public_key().clone(),
			0,
		);

		buckets[D - 1].remote_nodes.push(Arc::new(self_remote_node));

		KadDHT {
			local_node: Arc::new(local_node),
			remote_nodes: DashMap::new(),
			buckets,
			store: Box::new(MemoryStore::new()),
		}
	}

	// add a node, return the evicted node if any
	pub fn add_node(&mut self, node: RemoteNode) -> Option<EvictedNode> {
		// wrap the node in an Arc
		let node = Arc::new(node);

		// get the distance
		let distance = (node.public_key.u256()
			^ self.local_node.key_pair.public_key().u256())
		.leading_zeros_vartime();

		// if the bucket is not full, add the node
		if self.buckets[distance as usize].remote_nodes.len() < K {
			self.buckets[distance as usize]
				.remote_nodes
				.push(node.clone());
			self.remote_nodes
				.insert(node.public_key.u256().clone(), node.clone());
			return None;
		// otherwise, evict the node with the lowest ping if the new node has a lower ping
		} else {
			// find the node with the lowest ping
			let mut min_ping = u32::MAX;
			let mut min_ping_index = 0;
			for (i, n) in self.buckets[distance as usize]
				.remote_nodes
				.iter()
				.enumerate()
			{
				if n.ping_ms < min_ping {
					min_ping = n.ping_ms;
					min_ping_index = i;
				}
			}
			// test if the new node has a lower ping
			if node.ping_ms >= min_ping {
				return None;
			}

			// evict the node with the lowest ping
			let evicted_node = self.buckets[distance as usize]
				.remote_nodes
				.remove(min_ping_index);
			self.remote_nodes.remove(&evicted_node.public_key.u256());
			self.buckets[distance as usize]
				.remote_nodes
				.push(node.clone());
			self.remote_nodes
				.insert(node.public_key.u256().clone(), node.clone());
			return Some(evicted_node);
		}
	}

	// update a node (replace it if it exists), returns true if the node was updated
	pub fn update_node(&mut self, node: RemoteNode) -> bool {
		// wrap the node in an Arc
		let node = Arc::new(node);

		// get the distance
		let distance = (node.public_key.u256()
			^ self.local_node.key_pair.public_key().u256())
		.leading_zeros_vartime();

		// if the node exists, replace it
		if let Some(indexed_node) =
			self.remote_nodes.get(&node.public_key.u256())
		{
			self.buckets[distance as usize].remote_nodes.retain(|n| {
				n.public_key().u256() != indexed_node.public_key().u256()
			});
			self.buckets[distance as usize]
				.remote_nodes
				.push(node.clone());
			self.remote_nodes
				.insert(node.public_key.u256().clone(), node.clone());
			true
		} else {
			false
		}
	}

	// remove a node, returns true if the node was removed
	pub fn remove_node(&mut self, node: RemoteNode) -> bool {
		if let Some(indexed_node) =
			self.remote_nodes.get(&node.public_key.u256())
		{
			let distance = (node.public_key.u256()
				^ self.local_node.key_pair.public_key().u256())
			.leading_zeros_vartime();
			self.buckets[distance].remote_nodes.retain(|n| {
				n.public_key().u256() != indexed_node.public_key.u256()
			});
			self.remote_nodes.remove(&node.public_key.u256());
			return true;
		}
		false
	}

	// find the nearest nodes to a given key
	pub fn nearest_nodes(&self, key: U256) -> Vec<NearRemoteNode> {
		// Return vector
		let mut return_nodes = Vec::with_capacity(ALPHA);

		// Calculate the distance between the key and the local node's public key.
		let mut distance = (key ^ self.local_node.key_pair.public_key().u256())
			.leading_zeros_vartime();

		// Continue searching for nearest nodes until we have either:
		// - accumulated ALPHA nodes
		// - checked D buckets
		// - reached the 0th bucket
		while return_nodes.len() < ALPHA && distance != D {
			let mut potential_nodes =
				self.buckets[distance].remote_nodes.clone();

			while potential_nodes.len() != 0 && return_nodes.len() < ALPHA {
				let mut min_dist = usize::MAX;
				let mut min_dist_index = 0;
				for (i, n) in potential_nodes.iter().enumerate() {
					let dist =
						(n.public_key().u256() ^ key).leading_zeros_vartime();
					if dist < min_dist {
						min_dist = dist;
						min_dist_index = i;
					}
				}
				return_nodes.push(NearRemoteNode {
					distance: min_dist,
					remote_node: potential_nodes.remove(min_dist_index),
				});
			}

			// Move to the next bucket.
			distance += 1;
		}

		// Return the list of nearest nodes found.
		return_nodes
	}

	fn store_get(&self, key: U256) -> Option<Bytes> {
		self.store.get(key)
	}

	fn store_set(&self, key: U256, value: Bytes) {
		self.store.set(key, value);
	}

	fn get(&self, key: U256) -> Option<Bytes> {
		let nearest_nodes = self.nearest_nodes(key);
		//let results: Vec<Bytes> = Vec::with_capacity(ALPHA);

		for node in nearest_nodes.iter() {
			// if self is in the list of nearest nodes, try searching the store
			if node.remote_node.public_key().u256()
				== self.local_node.key_pair.public_key().u256()
			{
				if let Some(value) = self.store_get(key) {
					return Some(value);
				}
			}

			// otherwise, try searching the remote nodes (todo, rpc)
		}
		None
	}

	/**
	 * Display
		*/
	fn bucket_counts(&self) -> Vec<usize> {
		self.buckets.iter().map(|b| b.remote_nodes.len()).collect()
	}
}

const DUMMY_NODE: Node = Node {
	ipv4: Ipv4Addr::new(127, 0, 0, 1),
	ipv4_port: 1234,
	ipv6: None,
	ipv6_port: None,
};

/*
#[test]
fn test_kad_dht() -> Result<(), Box<dyn std::error::Error>> {


	const DUMMY_NODE_COUNT: usize = 20;
	let mut dummy_nodes = Vec::with_capacity(DUMMY_NODE_COUNT);

	// create remote nodes
	for _ in 0..DUMMY_NODE_COUNT {
		let key_pair = KeyPair::random()?;
		let remote_node = RemoteNode::new(
			Node {
				ipv4: Ipv4Addr::new(127, 0, 0, 1),
				ipv4_port: 1234,
				ipv6: None,
				ipv6_port: None,
			},
			key_pair.public_key().clone(),
			0,
		);
		dummy_nodes.push(remote_node);
	}

	let mut dht = KadDHT::new(LocalNode::new(DUMMY_NODE, KeyPair::random()?));

	// add the dummy nodes
	for node in dummy_nodes.iter() {
		dht.add_node(node.clone());
	}

	// check that the nodes were added
	println!("{:?}", dht.bucket_counts());


	const TRIALS: usize = 10;

	for _ in 0..TRIALS{

		// find nearest nodes to a random key
		let key = U256::random(&mut OsRng);

		// print the key distance
		println!("key distance to self: {:?}", (key ^ dht.local_node.key_pair.public_key().u256()).leading_zeros_vartime());

		let nearest_nodes = dht.nearest_nodes(key);

		// print the nearest nodes' public keys
		//println!("nearest node distances: {:?}", nearest_nodes.iter().map(|n| (n.remote_node().public_key().to_string(), n.distance())).collect::<Vec<_>>());
		println!("nearest node distances: {:?}", nearest_nodes.iter().map(|n| (n.distance())).collect::<Vec<_>>());
	}


	Ok(())
}
*/

/*
#[test]
fn test_kad_dht() -> Result<(), Box<dyn std::error::Error>> {


	const DUMMY_NODE_COUNT: usize = 20;
	let mut dummy_nodes = Vec::with_capacity(DUMMY_NODE_COUNT);

	// create remote nodes
	for _ in 0..DUMMY_NODE_COUNT {
		let key_pair = KeyPair::random()?;
		let remote_node = RemoteNode::new(
			Node {
				ipv4: Ipv4Addr::new(127, 0, 0, 1),
				ipv4_port: 1234,
				ipv6: None,
				ipv6_port: None,
			},
			key_pair.public_key().clone(),
			0,
		);
		dummy_nodes.push(remote_node);
	}

	let mut dht = KadDHT::new(LocalNode::new(DUMMY_NODE, KeyPair::random()?));

	// add the dummy nodes
	for node in dummy_nodes.iter() {
		dht.add_node(node.clone());
	}

	// check that the nodes were added
	println!("{:?}", dht.bucket_counts());


	const TRIALS: usize = 10;

	for _ in 0..TRIALS{

		// find nearest nodes to a random key
		let key = U256::random(&mut OsRng);

		// print the key distance
		println!("key distance to self: {:?}", (key ^ dht.local_node.key_pair.public_key().u256()).leading_zeros_vartime());

		let nearest_nodes = dht.nearest_nodes(key);

		// print the nearest nodes' public keys
		//println!("nearest node distances: {:?}", nearest_nodes.iter().map(|n| (n.remote_node().public_key().to_string(), n.distance())).collect::<Vec<_>>());
		println!("nearest node distances: {:?}", nearest_nodes.iter().map(|n| (n.distance())).collect::<Vec<_>>());
	}


	Ok(())
}
*/
