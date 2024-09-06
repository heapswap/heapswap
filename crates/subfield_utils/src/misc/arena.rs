pub use generational_arena::{Arena, Index as ArenaIndex};

/*
pub trait ArenaIndexExt64 {
	fn to_tuple(self) -> (u64, u64);
	fn from_tuple(tuple: (u64, u64)) -> Self;
	fn to_bytes(self) -> [u8; 16];
	fn from_bytes(bytes: &[u8; 16]) -> Self;
}

impl ArenaIndexExt64 for ArenaIndex {
	fn to_tuple(self) -> (u64, u64) {
		let (a, b) = self.into_raw_parts();
		(a as u64, b)
	}

	fn from_tuple(tuple: (u64, u64)) -> Self {
		let (a, b) = tuple;
		Self::from_raw_parts(a as usize, b)
	}

	fn to_bytes(self) -> [u8; 16] {
		let (a, b) = self.to_tuple();
		let mut bytes = [0u8; 16];
		bytes[..8].copy_from_slice(&a.to_be_bytes());
		bytes[8..].copy_from_slice(&b.to_be_bytes());
		bytes
	}

	fn from_bytes(bytes: &[u8; 16]) -> Self {
		let a = u64::from_be_bytes(bytes[..8].try_into().unwrap());
		let b = u64::from_be_bytes(bytes[8..].try_into().unwrap());
		Self::from_tuple((a, b))
	}
}


#[test]
fn test_arena_ext_64() {
	let mut arena = Arena::new();
	let key = arena.insert(1);
	let key2 = ArenaIndex::from_bytes(&key.to_bytes());
	assert_eq!(key, key2);
}
*/

pub type ArenaHandleTuple = (u32, u32);
pub type ArenaHandleBytes = [u8; 8];

pub trait ArenaIndexExt32 {
	fn to_tuple(self) -> (u32, u32);
	fn from_tuple(tuple: (u32, u32)) -> Self;
	fn to_bytes(self) -> [u8; 8];
	fn from_bytes(bytes: &[u8; 8]) -> Self;
}

impl ArenaIndexExt32 for ArenaIndex {
	fn to_tuple(self) -> (u32, u32) {
		let (a, b) = self.into_raw_parts();
		(a as u32, b as u32)
	}

	fn from_tuple(tuple: (u32, u32)) -> Self {
		let (a, b) = tuple;
		Self::from_raw_parts(a as usize, b as u64)
	}

	fn to_bytes(self) -> [u8; 8] {
		let (a, b) = self.to_tuple();
		let mut bytes = [0u8; 8];
		bytes[..4].copy_from_slice(&a.to_be_bytes());
		bytes[4..].copy_from_slice(&b.to_be_bytes());
		bytes
	}

	fn from_bytes(bytes: &[u8; 8]) -> Self {
		let a = u32::from_be_bytes(bytes[..4].try_into().unwrap());
		let b = u32::from_be_bytes(bytes[4..].try_into().unwrap());
		Self::from_tuple((a, b))
	}
}

#[test]
fn test_arena_ext_32() {
	let mut arena = Arena::new();
	let key = arena.insert(1);
	let key2 = ArenaIndex::from_bytes(&key.to_bytes());
	assert_eq!(key, key2);
}
