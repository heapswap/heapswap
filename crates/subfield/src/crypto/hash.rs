use subfield_proto::versioned_bytes::VersionedBytes;

use crate::arr;
use crate::versioned_bytes::*;

pub type Hash = VersionedBytes;

pub fn hash(data: &[u8]) -> Hash {
	VersionedBytes {
		version: 0,
		data: blake3::hash(data).as_bytes().to_vec(),
	}
}

pub fn hash_verify(data: &[u8], data_hash: Hash) -> bool {
	hash(data) == data_hash
}
