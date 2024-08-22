use std::{cmp::Reverse, collections::HashMap};

use super::*;
use crate::*;

#[derive(Clone)]
pub struct KadConfig {
	pub local_node: LocalNode,
	pub alpha: u8, // degree of network request parallelism
	pub beta: u8,  // number of closest buckets to query
	pub k: u8,     // number of buckets stored in each k-bucket
}

pub enum TryAddNodeResult {
	Added,
	Replaced(RemoteNode),
	NotAdded,
}

pub enum NearestNNodesResult {
	Found(Vec<RemoteNode>),
	FoundShouldContainSelf(Vec<RemoteNode>),
	NoneFound,
}

pub enum NearestNodeResult {
	Found(RemoteNode),
	SelfIsNearest,
	NoneFound,
}

#[derive(Clone, Getters)]
pub struct Kad {
	#[getset(get = "pub")]
	config: KadConfig,

	// 0 zeroes = opposite to self, 255 zeroes = self
	#[getset(get = "pub")]
	buckets: Vec<Vec<RemoteNode>>,
}

impl Kad {
	fn new(config: KadConfig) -> Self {
		Self {
			config,
			buckets: vec![vec![]; 256],
		}
	}

	fn calculate_distance(&self, key: &V256) -> u32 {
		self.config
			.local_node
			.public_key()
			.v256()
			.xor_leading_zeros(key)
	}

	// add a node - can either succeed, replace, or fail
	// TODO: don't evict the node with the highest ping, in favor of old nodes with higher uptimes
	fn try_add_node(&mut self, node: RemoteNode) -> TryAddNodeResult {
		let distance = self.calculate_distance(&node.public_key().v256());
		let mut bucket = self.buckets[distance as usize].clone();

		// check if the bucket has space
		if bucket.len() < self.config.k as usize {
			bucket.push(node);
			sort_bucket_by_ping_increasing(&mut bucket);
			self.buckets[distance as usize] = bucket;
			return TryAddNodeResult::Added;
		// check if the ping of the new node is less than the ping of the last node in the bucket
		} else if node.ping_ms() < bucket[bucket.len() - 1].ping_ms() {
			// the new node has a lower ping than the highest ping in the bucket
			bucket.push(node);
			sort_bucket_by_ping_increasing(&mut bucket);
			let (bucket, replaced_node) =
				bucket.split_at(self.config.k as usize);
			self.buckets[distance as usize] = bucket.to_vec();
			return TryAddNodeResult::Replaced(replaced_node[0].clone());
		} else {
			// the new node has a higher ping than the highest ping in the bucket
			return TryAddNodeResult::NotAdded;
		}
	}

	// remove a node - can either succeed, or fail
	fn try_remove_node(&mut self, node: RemoteNode) -> bool {
		let distance = self
			.config
			.local_node
			.public_key()
			.v256()
			.xor_leading_zeros(&node.public_key().v256());
		let mut bucket = self.buckets[distance as usize].clone();
		let bucket_len = bucket.len();
		bucket.retain(|n| n.public_key() != node.public_key());
		if bucket.len() == bucket_len {
			return false;
		} else {
			self.buckets[distance as usize] = bucket;
			return true;
		}
	}

	// find the n closest nodes to a given node
	fn nearest_n_nodes(&self, key: &V256, n: usize) -> NearestNNodesResult {
		let mut nearest_nodes = Vec::new();
		let mut nearest_includes_self = false;

		let distance = self.calculate_distance(key);

		// starting at bucket [distance], iterate towards bucket [255] until n nodes are found. if 0 is reached, flip the flag, but do not add self to the list
		for i in (distance..=255) {
			if i == 0 {
				nearest_includes_self = true;
				break;
			}
			let bucket = self.buckets[i as usize].clone();
			for j in 0..bucket.len() {
				nearest_nodes.push(bucket[j].clone());
				if nearest_nodes.len() == n {
					break;
				}
			}
		}

		// If no nodes were found, return NoneFound
		if nearest_nodes.len() == 0 {
			return NearestNNodesResult::NoneFound;
		}

		// If the node list would have contained the local node, return FoundShouldContainSelf. The actual self node is not included to avoid recursion
		if nearest_includes_self {
			return NearestNNodesResult::FoundShouldContainSelf(nearest_nodes);
		} else {
			return NearestNNodesResult::Found(nearest_nodes);
		}
	}

	// find only the nearest node to a given node
	fn find_nearest_node(&self, key: &V256) -> NearestNodeResult {
		match self.nearest_n_nodes(key, 1) {
			NearestNNodesResult::Found(nodes) => {
				NearestNodeResult::Found(nodes[0].clone())
			}
			NearestNNodesResult::FoundShouldContainSelf(nodes) => {
				NearestNodeResult::SelfIsNearest
			}
			NearestNNodesResult::NoneFound => NearestNodeResult::NoneFound,
		}
	}
}

fn sort_bucket_by_ping_increasing(bucket: &mut Vec<RemoteNode>) {
	bucket.sort_by(|a, b| a.ping_ms().cmp(&b.ping_ms()));
}

#[test]
fn test_kad() {
	let config = KadConfig {
		local_node: LocalNode::new(crypto::PrivateKey::random()),
		alpha: 3,
		beta: 3,
		k: 20,
	};
}