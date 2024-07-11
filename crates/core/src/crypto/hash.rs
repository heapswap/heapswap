use crate::misc::u256::U256;
use crate::{
	arr, bys,
	traits::{Arrable, Base32able, Byteable, Stringable},
};
use bytes::Bytes;

pub type Hash = [u8; 32];

impl Base32able<HashError> for Hash {
	fn from_base32(string: &str) -> Result<Self, HashError> {
		arr::from_base32(string).map_err(|_| HashError::InvalidBase32)
	}

	fn to_base32(&self) -> String {
		arr::to_base32(self)
	}
}

#[derive(Debug)]
pub enum HashError {
	InvalidBase32,
	InvalidHash,
}

pub fn hash(data: &[u8]) -> Hash {
	//Hash::from_le_bytes(blake3::hash(data.as_ref()).into())
	//Hash::from_arr(&blake3::hash(data.as_ref()).into()).unwrap()
	blake3::hash(data).into()
}

pub fn verify(data: &[u8], data_hash: Hash) -> bool {
	hash(data) == data_hash
}
