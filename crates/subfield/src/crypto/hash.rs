use crate::arr;
use crate::u256::*;

pub type Hash = U256;

pub fn hash(data: &[u8]) -> Hash {
	U256::hash(&data)
}

pub fn hash_verify(data: &[u8], data_hash: Hash) -> bool {
	hash(data) == data_hash
}
