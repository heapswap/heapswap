use crate::arr;
use crate::traits::*;
use bincode::{deserialize, serialize};
use crypto_bigint::Encoding;
use crypto_bigint::Random;
use crypto_bigint::U256;
use rand::rngs::OsRng;

#[derive(Debug, strum::Display)]
pub enum U256Error {
	InvalidBase32,
}

impl Base32able<U256Error> for U256 {
	fn to_base32(&self) -> String {
		arr::to_base32(&self.to_le_bytes())
	}

	fn from_base32(string: &str) -> Result<Self, U256Error> {
		arr::from_base32(string)
			.map(|arr| U256::from_le_bytes(arr.try_into().unwrap()))
			.map_err(|_| U256Error::InvalidBase32)
	}
}

#[test]
fn test_u256() {
	let random_u256 = U256::random(&mut OsRng);

	let random_u256_encoded = random_u256.to_base32();
	let random_u256_decoded = U256::from_base32(&random_u256_encoded).unwrap();
	assert_eq!(random_u256, random_u256_decoded);

	let random_u256_serialized = serialize(&random_u256).unwrap();
	assert_eq!(random_u256_serialized.len(), 32);

	let random_u256_deserialized: U256 =
		deserialize(&random_u256_serialized).unwrap();
	assert_eq!(random_u256, random_u256_deserialized);
}
