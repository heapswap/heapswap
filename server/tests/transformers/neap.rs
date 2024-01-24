use ort::value;
use rand::Rng;

/*
	neap ^◡^
	(node heap)

	rules:
		- either left and right are both None, or both Some
		- left is a copy of itself
		- right is always greater than left

*/
/*
#[derive(Clone)]
struct KVNode<K, V> {
	key: K,
	value: V,
	left: Option<Box<KVNode<K, V>>>,
	right: Option<Box<KVNode<K, V>>>,
	right_distance: i64,
}

impl<K, V> KVNode<K, V> {
	// constructor
	fn new(
		key: K,
		value: V,
	) -> Self {
		Self {
			key,
			value,
			left: None,
			right: None,
			right_distance: 0,
		}
	}

}

type Node = KVNode<i64, String>;

impl Node {
	// get the distance between the key and the node's key
	fn distance(
		&self,
		key: i64,
	) -> i64 {
		(key ^ self.key).count_ones() as i64
	}
	
	// insert a new node into the neap
	fn insert(
		&mut self,
		key: i64,
		value: String,
	) -> &mut Node{
		// if the key is the same as the node's key, update the value
		if self.key == key {
			self.value = value.clone();
			
			// if there are children, update their values too
			if let Some(left) = self.left.as_mut() {
				return left.insert(key, value);
			} else {				
				// otherwise return self
				return self;
			}			
		
		} 
		
		// if the key is different, insert it into the neap
		
		// if the node has children
		if let Some(left) = self.left.as_mut() {
			
			
			if left.distance(key) < self.right_distance {
				return left.insert(key, value);
			} else {
				return self.right.as_mut().insert(key, value);
			} 
		} else {
			self.left = Some(Box::new(Node::new(self.key, self.value.clone())));
			self.right = Some(Box::new(Node::new(key, value)));
			self.right_distance = self.distance(key);
		}
	}

	// get a node from the neap
	fn get(
		&self,
		key: i64,
	) -> Option<Node> {
		if self.key == key {
			// found it
			return Some(self.clone());
		} else if self.left.is_none() {
			// not found
			return None;
		} else if self.distance(key) <= self.right_distance {
			// distance to left is less than distance to right
			return self.left.as_ref().unwrap().get(key);
		} else {
			// distance to right is more than distance to left
			return self.right.as_ref().unwrap().get(key);
		}
	}
}
*/
#[test]
fn main() {
		let mut nodes = vec![
			0b0000, 
			//0b0001, 
			0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111,
			0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110, 0b1111,
		];
		nodes.sort();
		
		let goal = 0b0001;
		
		let mut low = 0;
		let mut high = nodes.len() - 1;
	
		while low <= high {
			let mid = low + (high - low) / 2;
			if nodes[mid] == goal {
				println!("Found goal at index: {}", mid);
				return;
			}
			if nodes[mid] < goal {
				low = mid + 1;
			} else {
				high = mid - 1;
			}
		}
		
		// high and low switch when they are not found
		println!("Goal not found - low: {}, high: {}", high, low);
}