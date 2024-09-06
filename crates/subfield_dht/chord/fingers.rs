use crate::*;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::net::{Ipv4Addr, Ipv6Addr};

pub type ChordId = PublicKey;

const KEYSPACE_SIZE: usize = 256;

lazy_static! {
	static ref KEYSPACE_SIZE_BIGINT: BigUint = BigUint::from(256u32);
    static ref _2_TO_KEYSPACE_SIZE: BigUint = BigUint::from(2u32).pow(KEYSPACE_SIZE as u32);
	static ref _2K_MINUS_ONE_LOOKUP: Vec<BigUint> = (1..=KEYSPACE_SIZE).map(|k| BigUint::from(2u32).pow(k as u32)).collect();
}

pub fn k_to_finger_index(n: &ChordId, k: u16) -> ChordId {
	let _2k_minus_1 = &_2K_MINUS_ONE_LOOKUP[k as usize - 1];
	let index = (n.bigint() + _2k_minus_1) % _2_TO_KEYSPACE_SIZE.clone();
	ChordId::new(V256::from_bigint(index))
}

/*
pub struct RemoteChordNode {
	pub id: ChordId,
	pub ip_address: Either<Ipv4Addr, Ipv6Addr>,
	pub ip_port: u16,
}

impl RemoteChordNode {
	pub fn new(id: ChordId, ip_address: Either<Ipv4Addr, Ipv6Addr>, ip_port: u16) -> Self {
		Self { id, ip_address, ip_port }
	}
	
	pub fn random() -> Self {
		let id = ChordId::random();
		let ip_address = Either::Left(Ipv4Addr::random());
		let ip_port = random();
		Self::new(id, ip_address, ip_port)
	}
}


#[derive(Getters, Setters)]
pub struct LocalChordNode {
	
	remote: RemoteChordNode,
	
	keypair: Keypair,
	
	pub predecessor: Option<ChordId>,
	pub successor: Option<ChordId>,
	// pub fingers: [ChordId; KEYSPACE_SIZE],
	pub fingers: OrderedMap<ChordId, Option<RemoteChordNode>>,
}






impl LocalChordNode {
	pub fn new(keypair: Keypair, ip_address: Either<Ipv4Addr, Ipv6Addr>, ip_port: u16) -> Self {
		
		let remote = RemoteChordNode {
			id: keypair.public_key().clone(),
			ip_address,
			ip_port,
		};
		
		let fingers = Self::build_fingers(&remote.id);

		Self { remote, keypair, predecessor: None, successor: None, fingers }
	}
	
	pub fn random() -> Self {
		let keypair = Keypair::random();
		let ip_address = Either::Left(Ipv4Addr::random());
		let ip_port = random();
		Self::new(keypair, ip_address, ip_port)
	}

	fn build_fingers(id: &ChordId) -> OrderedMap<ChordId, Option<RemoteChordNode>> {
		let mut fingers = OrderedMap::new();
		for i in 0..KEYSPACE_SIZE {
			let finger = k_to_finger_index(id, i as u8);
			fingers.insert(finger.clone(), None);
		}
		fingers
	}

	fn find_successor(&self, id: &ChordId) -> Option<&RemoteChordNode> {
		self.fingers.successor(id).and_then(|node| node.as_ref())
	}

	fn find_predecessor(&self, id: &ChordId) -> Option<&RemoteChordNode> {
		self.fingers.predecessor(id).and_then(|node| node.as_ref())
	}
}







#[test]
pub fn test_node_creation() {
	let local_node = LocalChordNode::random();

	let key = ChordId::random();

	// find successor
	let successor = local_node.find_successor(&key);
	assert!(successor.unwrap().id > key);

	// find predecessor
	let predecessor = local_node.find_predecessor(&key);
	assert!(predecessor.unwrap().id < key);
}
*/