use crate::{
	bys,
	traits::{Arrable, Byteable, Stringable},
};
use bytes::Bytes;
//use crypto_bigint::{Encoding, U256};
use crate::misc::u256::U256;

pub type Hash = U256;

#[derive(Debug)]
pub enum HashError {
	InvalidHash,
}

pub trait Hashing {
	fn hash(data: &[u8]) -> Hash;
	fn verify(&self, data: &[u8]) -> bool;
}

impl Hashing for Hash {
	fn hash(data: &[u8]) -> Hash {
		//Hash::from_le_bytes(blake3::hash(data.as_ref()).into())
		Hash::from_arr(&blake3::hash(data.as_ref()).into()).unwrap()
	}

	fn verify(&self, data: &[u8]) -> bool {
		self == &Hash::hash(data)
	}
}
