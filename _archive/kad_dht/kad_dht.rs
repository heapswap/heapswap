use super::node::*;

const BUCKET_COUNT: usize = 8;

struct Bucket {
	count: usize,
	nodes: Vec<RemoteNode>,
}

impl Bucket {
	fn new(count: usize) -> Self {
		Bucket {
			count,
			nodes: Vec::new(),
		}
	}

	fn add(&mut self, node: RemoteNode) {
		self.nodes.push(node);
	}

	fn remove(&mut self, node: &RemoteNode) {
		self.nodes.retain(|n| n.public_key().as_u256() != node.public_key().as_u256());
	}

	fn iter(&self, node: &RemoteNode) -> Option<&RemoteNode> {
		self.nodes.iter().find(|n| n.public_key().as_u256() == node.public_key().as_u256())
	}
}