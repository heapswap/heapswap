use crate::*;

const KEYSPACE_SIZE: usize = 256;

lazy_static! {
	static ref KEYSPACE_SIZE_BIGINT: BigUint = BigUint::from(KEYSPACE_SIZE as u32);
    static ref _2_TO_KEYSPACE_SIZE: BigUint = BigUint::from(2u32).pow(KEYSPACE_SIZE as u32);
	static ref _2K_LOOKUP: Vec<BigUint> = (1..=KEYSPACE_SIZE).map(|k| BigUint::from(2u32).pow(k as u32)).collect();
}

#[wasm_bindgen]
pub struct ChordNode{
	pub id: ChordId,
	pub node: JsValue,
}

#[wasm_bindgen]
pub enum ChordNodeType{
	Local(ChordNode),
	Remote(ChordNode),
	None
}

#[wasm_bindgen]
pub enum TryAddChordNodeResult{
	DidNotAdd,
	Added,
	AddedWithEviction(ChordNode),
	CannotAddSelf,
}

type ChordId = VersionedBytes;
type FingerId = VersionedBytes;


#[wasm_bindgen]
pub struct ChordMap {
	self_node: ChordNode,
	
	// finger lookup map (including self)
	finger_map: OrderedMap<FingerId, ChordNodeType>,
	
	// finger vec (excludes self)
	finger_vec: Vec<FingerId>,
}


#[wasm_bindgen]
impl ChordMap {
	
	/*
		Constructors
	*/
	
	#[wasm_bindgen(constructor)]
	pub fn new(self_node: ChordNode) -> Self {
		let finger_map = OrderedMap::new();
		let finger_vec = Vec::with_capacity(KEYSPACE_SIZE);
		finger_map.insert(self_node.id.clone(), ChordNodeType::Local(self_node.clone()));
		
		for i in 1..=KEYSPACE_SIZE {
			let finger = k_to_finger_index(id, i as u16);
			finger_map.insert(finger.clone(), ChordNodeType::None);
			finger_vec.push(finger);
		}
		
		Self { self_node, finger_map, finger_vec }
	}
	
	/*
		Getters
	*/
	pub fn fingers(&self) -> &Vec<FingerId> {
		&self.finger_vec
	}
	
	
	/*
		Add/Remove
	*/
	#[wasm_bindgen]
	pub fn try_add(&self, maybe_new_node: ChordNode) -> TryAddChordNodeType {
		// find successor finger
		let (finger_id, finger_node) = self.finger_map.successor(key).unwrap();
		
		match finger_node {
			ChordNodeType::Local(finger_node) => {
				// local node will never try to be added
				TryAddChordNodeResult::CannotAddSelf
			},
			ChordNodeType::Remote(finger_node) => {
				// node.set(key, chord_node);
				if maybe_new_node.id < finger_node.id {
					// evict the finger node
					self.finger_map.insert(finger_id, ChordNodeType::Remote(maybe_new_node));
					TryAddChordNodeResult::AddedWithEviction(finger_node)
				} else {
					TryAddChordNodeResult::DidNotAdd
				}
			},
			ChordNodeType::None => {
				// no node exists, add it 
				self.finger_map.insert(finger_id, ChordNodeType::Remote(maybe_new_node));
				TryAddChordNodeResult::Added
			}
		}	
	}
	
	#[wasm_bindgen]
	pub fn try_remove(&self, maybe_node_to_remove_id: ChordId) -> Option<ChordNode> {
		// find successor finger
		let (finger_id, finger_node) = self.finger_map.successor(maybe_node_to_remove_id).unwrap();
		// if maybe_node_to_remove_id and finger_node are the same, then remove the finger node
		if maybe_node_to_remove_id == finger_node.id {
			self.finger_map.insert(finger_id, ChordNodeType::None);
			Some(finger_node)
		} else {
			None
		}
	}
	
	
	/*
		Successor/Predecessor
	*/
	#[wasm_bindgen]
	pub fn successor(&self, key: &FingerId) -> &ChordNodeType {
		self.finger_map
			.range((std::ops::Bound::Included(key), std::ops::Bound::Unbounded))
			.find(|(_, v)| !matches!(v, ChordNodeType::None))
			.map(|(_, v)| v)
			.or_else(|| self.finger_map.chord_nodes().find(|v| !matches!(v, ChordNodeType::None))) // Wrap around
			.unwrap_or(ChordNodeType::None) // this will never happen, since self is always in the map
	}
	
	#[wasm_bindgen]
	pub fn predecessor(&self, key: &FingerId) -> &ChordNodeType {
		self.finger_map
			.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(key)))
			.rev()
			.find(|(_, v)| !matches!(v, ChordNodeType::None))
			.map(|(_, v)| v)
			.or_else(|| self.finger_map.chord_nodes().rev().find(|v| !matches!(v, ChordNodeType::None))) // Wrap around
			.unwrap_or(ChordNodeType::None) // this will never happen, since self is always in the map
	}

	
	#[wasm_bindgen]
	pub fn self_predecessor(&self) -> &ChordNode {
		let pre = self.predecessor(&self.self_node.id);
		match pre {
			ChordNodeType::Remote(pre) => pre,
			ChordNodeType::Local(pre) => panic!("self_predecessor should never be local"),
			ChordNodeType::None => panic!("self_predecessor should never be None"),
		}
	}
}