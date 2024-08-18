use crate::arr;
use crate::versioned_bytes::*;
use crate::*;

pub type Hash = V256;

pub fn hash(data: &[u8]) -> Hash {
	V256::new(
		0,
		blake3::hash(data).as_bytes().to_vec().try_into().unwrap(),
	)
}

pub fn hash_verify(data: &[u8], data_hash: Hash) -> bool {
	hash(data) == data_hash
}
