use crate::{
	bys,
	traits::{Arrable, Byteable, Stringable},
};
use bytes::Bytes;
use crypto_bigint::{Encoding, U256};

type Hash = U256;

#[derive(Debug)]
pub enum HashError {
	InvalidHash,
}

pub trait Hashing {
	fn hash(data: Bytes) -> Hash;
	fn verify(&self, data: Bytes) -> bool;
}

impl Hashing for Hash {
	fn hash(data: Bytes) -> Hash {
		Hash::from_le_bytes(blake3::hash(data.as_ref()).into())
	}

	fn verify(&self, data: Bytes) -> bool {
		self == &Hash::hash(data)
	}
}
