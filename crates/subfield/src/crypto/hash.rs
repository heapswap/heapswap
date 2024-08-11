use crate::arr;
use crate::vector::*;

pub type Hash = U256;

pub fn hash(data: &[u8]) -> Hash {
	U256::from_u8(*blake3::hash(data).as_bytes())
}

pub fn verify_hash(data: &[u8], data_hash: Hash) -> bool {
	hash(data) == data_hash
}
