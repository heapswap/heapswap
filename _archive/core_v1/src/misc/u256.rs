use crate::arr;
use crate::traits::*;
use bincode::{deserialize, serialize};
use bytes::Bytes;
use crypto_bigint::Encoding;
use crypto_bigint::Random;
//use crypto_bigint::U256;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::BitXor;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U256 {
	unpacked: [u8; 32],
}

impl Serialize for U256 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let string_repr = self.to_base32();
		serializer.serialize_str(&string_repr)
	}
}

impl<'de> Deserialize<'de> for U256 {
	fn deserialize<D>(deserializer: D) -> Result<U256, D::Error>
	where
		D: Deserializer<'de>,
	{
		let string_repr = String::deserialize(deserializer)?;
		U256::from_base32(&string_repr).map_err(serde::de::Error::custom)
	}
}

#[derive(Debug, strum::Display)]
pub enum U256Error {
	InvalidBase32,
	InvalidLength,
}

impl Byteable<U256Error> for U256 {
	fn to_bytes(&self) -> Bytes {
		//Bytes::from(self.to_le_bytes().to_vec())
		Bytes::from(self.to_arr().to_vec())
	}

	fn from_bytes(bytes: &Bytes) -> Result<Self, U256Error> {
		Ok(U256::from_arr(bytes.as_ref().try_into().unwrap()).unwrap())
	}
}

impl Base32able<U256Error> for U256 {
	fn to_base32(&self) -> String {
		arr::to_base32(&self.to_arr())
	}

	fn from_base32(string: &str) -> Result<Self, U256Error> {
		let _arr: [u8; 32] = arr::from_base32(string)
			.map_err(|_| U256Error::InvalidBase32)?
			.try_into()
			.unwrap();
		if _arr.len() != 32 {
			return Err(U256Error::InvalidLength);
		}
		Ok(U256::from_arr(&_arr).unwrap())
	}
}

impl Arrable<[u8; 32], U256Error> for U256 {
	fn to_arr(&self) -> [u8; 32] {
		//self.to_le_bytes()
		self.unpacked
	}

	fn from_arr(arr: &[u8; 32]) -> Result<Self, U256Error> {
		//Ok(U256::from_le_bytes(*arr))
		Ok(U256 { unpacked: *arr })
	}
}

impl Stringable<U256Error> for U256 {
	fn to_string(&self) -> String {
		self.to_base32()
	}

	fn from_string(string: &str) -> Result<Self, U256Error> {
		U256::from_base32(string)
	}
}

impl Randomable for U256 {
	fn random() -> Self {
		let mut rng = OsRng;
		let mut arr = [0u8; 32];
		rng.fill_bytes(&mut arr);
		U256::from_arr(&arr).unwrap()
	}
}

#[test]
fn test_u256() {
	let random_u256 = U256::random();

	let random_u256_encoded = random_u256.to_base32();
	let random_u256_decoded = U256::from_base32(&random_u256_encoded).unwrap();

	assert_eq!(random_u256, random_u256_decoded);

	let random_u256_serialized = serialize(&random_u256).unwrap();
	//assert_eq!(random_u256_serialized.len(), 32);

	let random_u256_deserialized: U256 =
		deserialize(&random_u256_serialized).unwrap();
	assert_eq!(random_u256, random_u256_deserialized);
}
