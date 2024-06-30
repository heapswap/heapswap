use super::{LocalNode, Node, RemoteNode};
use crate::comparison::{hamming, xor};
use crate::constants::NS;
use crate::crypto::keys::KeyPair;
use crate::{crypto::keys::KeyArr, u256::*};
use crate::misc::traits::*;
use crate::u256::hamming256;
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;

pub struct DHT {
    max_dist_nodes: u32,
    max_ping_nodes: u32,

    local_node: LocalNode,

    // the remote_node in this is the source of truth and should be used to update the other two
    remote_nodes: HashMap<KeyArr, RemoteNode>,
    remote_nodes_by_dist: PriorityQueue<KeyArr, u32>,
    remote_nodes_by_ping: PriorityQueue<KeyArr, u32>,
}

type EvictedNode = RemoteNode;

pub struct NearestNodeResult{
	nearest_node: RemoteNode,
	dist: u32,
}

impl DHT {
    fn new(local_node: LocalNode, max_dist_nodes: u32, max_ping_nodes: u32) -> DHT {
        DHT {
            max_dist_nodes,
            max_ping_nodes,
            local_node,
            remote_nodes: HashMap::new(),
            remote_nodes_by_dist: PriorityQueue::new(),
            remote_nodes_by_ping: PriorityQueue::new(),
        }
    }

	
	/**
	 * Add Node
	*/
    fn add_remote_node_by_dist(&mut self, remote_node: RemoteNode) {
        let key_arr = remote_node.public_key.to_arr();

        // Add to remote_nodes_by_dist
        self.remote_nodes_by_dist
            .push(key_arr.clone(), remote_node.dist_to_self);

        // Add to remote_nodes
        self.remote_nodes.insert(key_arr, remote_node);
    }

    fn add_remote_node_by_ping(&mut self, remote_node: RemoteNode) {
        let key_arr = remote_node.public_key.to_arr();

        // Add to remote_nodes_by_dist
        self.remote_nodes_by_ping
            .push(key_arr.clone(), remote_node.ping_ms);

        // Add to remote_nodes
        self.remote_nodes.insert(key_arr, remote_node);
    }

	/**
	 * Update Node
	*/
    fn update_remote_node_by_dist(&mut self, remote_node: RemoteNode) {
        let key_arr = remote_node.public_key.to_arr();

        // Modify remote_nodes_by_dist
        self.remote_nodes_by_dist
            .change_priority(&key_arr, remote_node.dist_to_self);

        // Modify remote_nodes
        self.remote_nodes.insert(key_arr, remote_node);
    }
	
	fn update_remote_node_by_ping(&mut self, remote_node: RemoteNode) {
		let key_arr = remote_node.public_key.to_arr();

		// Modify remote_nodes_by_dist
		self.remote_nodes_by_ping
			.change_priority(&key_arr, remote_node.ping_ms);

		// Modify remote_nodes
		self.remote_nodes.insert(key_arr, remote_node);
	}

	
	/**
	 * Remove Node
	*/
    fn remove_remote_node_by_dist(&mut self, key_arr: &KeyArr) -> Option<EvictedNode> {
        // Remove from remote_nodes_by_dist
        self.remote_nodes_by_dist.remove(key_arr);

        // Remove from remote_nodes
        let evicted = self.remote_nodes.remove(key_arr);

        evicted
    }

    fn remove_remote_node_by_ping(&mut self, key_arr: &KeyArr) -> Option<EvictedNode> {
        // Remove from remote_nodes_by_dist
        self.remote_nodes_by_ping.remove(key_arr);

        // Remove from remote_nodes
        let evicted = self.remote_nodes.remove(key_arr);

        evicted
    }

	/**
	 * Try Add Node
	*/
    pub fn try_add_node(&mut self, remote_node: RemoteNode) -> Option<EvictedNode> { 
    //    self.try_evict(remote_node.clone(), true)
    //        .or_else(|| self.try_evict(remote_node, false))
    //}

    //fn try_evict(&mut self, remote_node: RemoteNode, is_testing_dist: bool) -> Option<EvictedNode> {
        //let (queue, max_queue_len) = if is_testing_dist {
        //    (&mut self.remote_nodes_by_dist, self.max_dist_nodes)
        //} else {
        //    (&mut self.remote_nodes_by_ping, self.max_ping_nodes)
        //};

        let key_arr = remote_node.public_key.to_arr();

		
        // Return None if the node is already in the DHT
        if self.remote_nodes.contains_key(&key_arr) {
            return None;
        }

		/*
        // Determine if eviction is necessary
        let should_evict = queue.len() as u32 >= max_queue_len
            && queue.peek().map_or(false, |(_, &farthest_dist)| {
                remote_node.dist_to_self < farthest_dist
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
		
		// Return None if the node is already in the DHT
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
			if let Some((&farthest_key, &farthest_dist)) = self.remote_nodes_by_dist.peek() {
				if remote_node.dist_to_self < farthest_dist {
					
					// remove the farthest node
					let evicted = self.remove_remote_node_by_dist(&farthest_key);
					
					// add the new node
					self.add_remote_node_by_dist(remote_node.clone());
					
					// try to add evicted to ping
					if let Some(evicted) = evicted {
						return self.try_add_node(evicted)
					} else {
						return None
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
			if let Some((&farthest_key, &farthest_ping)) = self.remote_nodes_by_ping.peek() {
				if remote_node.ping_ms < farthest_ping {
					// remove the farthest node
					let evicted = self.remove_remote_node_by_ping(&farthest_key);
					
					// add the new node
					self.add_remote_node_by_ping(remote_node);
					
					return evicted
				}
			}
		} else {
			// If the ping queue is not full, add the node
			self.add_remote_node_by_ping(remote_node);
			return None;
		}
		
		None
    }
	
	/**
	 * Nearest N Nodes to Self
	*/
	
	pub fn nearest_n_nodes_to_self_by_dist(&self, n: usize) -> Vec<RemoteNode> {
		self.remote_nodes_by_dist
			.iter()
			.take(n)
			.map(|(key, _)| self.remote_nodes.get(key).unwrap().clone())
			.collect()
	}

	pub fn nearest_n_nodes_to_self_by_ping(&self, n: usize) -> Vec<RemoteNode> {
		self.remote_nodes_by_ping
			.iter()
			.take(n)
			.map(|(key, _)| self.remote_nodes.get(key).unwrap().clone())
			.collect()
	}
	
	
	/**
	 * Absolute Nearest Node by dist to a KeyArr
	*/
	pub fn nearest_node_by_dist(&self, key_arr: &KeyArr) -> Option<NearestNodeResult> {
		
		let key_256 = U256::from_arr(key_arr).unwrap();
		
		let nearest_node = self.remote_nodes
			//.keys()
			//.min_by_key(|key| hamming(*key, key_arr));
			.iter()
			//.min_by_key(|(key, _)| hamming(*key, key_arr));
			.min_by_key(|(_, remote_node)| hamming256(&remote_node.public_key.as_u256(), &key_256));
			
		
		match nearest_node {
			//Some(key) => {
			Some((_, nearest_node)) => {
				let dist = hamming256(&nearest_node.public_key.as_u256(), &key_256);
				//let nearest_node = self.remote_nodes.get(key).unwrap();
				//let dist = hamming(key, key_arr);
				Some(NearestNodeResult { nearest_node : nearest_node.clone(), dist })
			},
			None => None
		}
	}

}


enum DHTError {
	InvalidKeyArr,
	InvalidKeyPair,
}

#[test]
fn test_dht() -> Result<(),()> {
	let dummy_node = Node {
		ipv4: Ipv4Addr::new(127, 0, 0, 1),
		ipv4_port: 1234,
		ipv6: None,
		ipv6_port: None,
	};
	
	let local_node = LocalNode::new(dummy_node.clone(), KeyPair::random().unwrap());
	
	let remote_node_count = 64;
	let mut remote_nodes = Vec::new();
	for _ in 0..remote_node_count {
		
		//let ping = i;
		// ping is random between 0 and 1000
		let ping = rand::random::<u32>() % 1000;
		let remote_node = RemoteNode::new(dummy_node.clone(), KeyPair::random().unwrap().public_key().clone(), ping, &local_node);
		remote_nodes.push(remote_node);
	}
	
	
	let mut dht = DHT::new(local_node, 10, 10);
	
	for remote_node in remote_nodes {
		let _ = dht.try_add_node(remote_node.clone());
	}
	
	println!("added nodes lens {}/{}", dht.remote_nodes_by_dist.len(), dht.remote_nodes_by_ping.len());
	
	
	// get closest 5 nodes to self
	let _ = dht.nearest_n_nodes_to_self_by_dist(5);
	let _ = dht.nearest_n_nodes_to_self_by_ping(5);
	
	// find closest node to a random key
	let random_key = KeyPair::random().unwrap().public_key().to_arr();
	let _ = dht.nearest_node_by_dist(&random_key).unwrap();
	
	
	/*
	// nearest node timing
	let s = timeit::timeit_loops!(1000,{
		let _ = dht.nearest_node_by_dist(&random_key).unwrap();
	});
	
	
	// timing (20 nodes)
	// v1: 13000ns/loop
	// v2: 7500ns/loop (using bytes, 300ns in release mode)
	// v3: 1700ns/loop (using u256, 81ns in release mode, 4ns per comparison)
	let nearest_node_result = dht.nearest_node_by_dist(&random_key).unwrap();
	println!("nearest node dist: {:?} ({}ns/loop)", nearest_node_result.dist, s * NS as f64);
	
	
	
	// idealized
	
	let random_256 = U256::random().unwrap();	
	
	let s = timeit::timeit_loops!(1000,{
		for _ in 0..dht.remote_nodes.len() {
			let _ = hamming256(&random_256, &random_256);
		}
	});
	
	
	// v1: 270ns/loop
	println!("idealized (u256): {}ns/loop", s * NS as f64);
	
	
	// idealized - bytes

	let random_256 = U256::random().unwrap().to_arr();

	let s = timeit::timeit_loops!(1000,{
		for _ in 0..dht.remote_nodes.len() {
			let _ = hamming(&random_256, &random_256);
		}
	});
	
	
	// v1: 6000ns/loop
	println!("idealized (bytes): {}ns/loop", s * NS as f64);
	
	*/
	
	Ok(())
}