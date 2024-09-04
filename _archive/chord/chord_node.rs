use crate::*;
use num_bigint::BigUint;
use num_traits::ToPrimitive;

pub type ChordId = V256;

const KEYSPACE_SIZE: usize = 256;

lazy_static! {
	static ref KEYSPACE_SIZE_BIGINT: BigUint = BigUint::from(256u32);
    static ref _2_TO_KEYSPACE_SIZE: BigUint = BigUint::from(2u32).pow(KEYSPACE_SIZE as u32);
	static ref _2K_LOOKUP: Vec<BigUint> = (0..KEYSPACE_SIZE).map(|k| BigUint::from(2u32).pow(k as u32)).collect();
}

pub struct ChordNode {
	pub id: ChordId,
	pub predecessor: Option<ChordId>,
	pub successor: Option<ChordId>,
	// pub fingers: [ChordId; KEYSPACE_SIZE],
	pub fingers: OrderedMap<ChordId, ChordId>,
}

impl ChordNode {
	pub fn new(id: ChordId) -> Self {
		// let mut fingers: [ChordId; KEYSPACE_SIZE] = (1..=KEYSPACE_SIZE).map(|i| ChordId::zeros_256(0)).collect::<Vec<ChordId>>().try_into().unwrap();
		
		let fingers = Self::build_fingers(&id);
		
		Self { id, predecessor: None, successor: None, fingers }
	}
	
	fn build_fingers(id: &ChordId) -> OrderedMap<ChordId, ChordId> {
		let mut fingers = OrderedMap::new();
		for i in 0..KEYSPACE_SIZE {
			let finger = k_to_finger_index(id, i as u8);
			fingers.insert(finger.clone(), finger);
		}
		fingers
	}
	
	fn find_successor(&self, id: &ChordId) -> &ChordId {
		self.fingers.successor(id).unwrap()
	}
	
	fn find_predecessor(&self, id: &ChordId) -> &ChordId {
		self.fingers.predecessor(id).unwrap()
	}
}



pub fn k_to_finger_index(n: &ChordId, k: u8) -> ChordId {
	let _2k_minus_1 = &_2K_LOOKUP[k as usize];
	let index = (n.bigint() + _2k_minus_1) % _2_TO_KEYSPACE_SIZE.clone();
	ChordId::from_bigint(index)
}



#[test]
pub fn test_node_creation() {
	let node = ChordNode::new(ChordId::random256());
	
	let key = ChordId::random256();
	
	// find successor
	let successor = node.find_successor(&key);
	assert!(successor.bigint() > key.bigint());
	
	// find predecessor
	let predecessor = node.find_predecessor(&key);
	assert!(predecessor.bigint() < key.bigint());
}