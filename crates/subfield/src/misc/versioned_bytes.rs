use std::hash::Hash;

use crate::*;

pub type V96 = VersionedBytes<12>;
pub type V256 = VersionedBytes<32>;
pub type V512 = VersionedBytes<64>;

pub trait HasV256 {
	fn v256(&self) -> &V256;
}

#[derive(Debug, strum::Display)]
pub enum VersionedBytesError {
	InvalidBase32,
	InvalidVersion,
}

#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct VersionedBytes<const DIM: usize> {
	#[getset(get = "pub")]
	version: u16,
	#[getset(get = "pub")]
	#[serde(with = "serde_bytes")]
	bytes: [u8; DIM],
	#[serde(skip)]
	string: OnceCell<String>,
}

impl<const DIM: usize> VersionedBytes<DIM> {
	pub fn new(version: u16, bytes: [u8; DIM]) -> Self {
		Self {
			version,
			bytes,
			string: OnceCell::new(),
		}
	}

	pub fn leading_zeros(&self) -> u32 {
		let mut count = 0;
		for i in 0..DIM {
			if self.bytes[i] == 0 {
				count += 8;
			} else {
				count += self.bytes[i].leading_zeros();
				break;
			}
		}
		count
	}

	pub fn xor_leading_zeros(&self, other: &Self) -> u32 {
		let mut count = 0;
		for i in 0..DIM {
			let xor = self.bytes[i] ^ other.bytes[i];
			if xor == 0 {
				count += 8;
			} else {
				count += xor.leading_zeros();
				break;
			}
		}
		count
	}
}

/**
 * Stringable
*/
impl<const DIM: usize> Stringable<VersionedBytesError> for VersionedBytes<DIM> {
	fn to_string(&self) -> String {
		self.string
			.get_or_init(|| arr::to_base32((&self.to_vec()).as_ref()))
			.clone()
	}

	fn from_string(s: &str) -> Result<Self, VersionedBytesError> {
		let vec = arr::from_base32(s)
			.map_err(|_| VersionedBytesError::InvalidBase32)?;
		let arr: [u8; DIM] = vec
			.as_slice()
			.try_into()
			.map_err(|_| VersionedBytesError::InvalidVersion)?;
		VersionedBytes::from_arr(&arr)
	}
}
impl<const DIM: usize> Into<String> for VersionedBytes<DIM> {
	fn into(self) -> String {
		self.to_string()
	}
}

/**
 * Vecable
*/
impl<const DIM: usize> Vecable<VersionedBytesError> for VersionedBytes<DIM> {
	fn from_arr(arr: &[u8]) -> Result<Self, VersionedBytesError> {
		let (bytes, version) = arr.split_at(arr.len() - 2);
		let version = u16::from_le_bytes(version.try_into().unwrap());
		Ok(VersionedBytes::new(version, bytes.try_into().unwrap()))
	}

	fn to_vec(&self) -> Vec<u8> {
		let version_bytes: [u8; 2] = self.version.to_le_bytes();
		[self.bytes.as_slice(), &version_bytes].concat()
	}
}

/**
 * Byteable
*/
impl<const DIM: usize> Byteable<VersionedBytesError> for VersionedBytes<DIM> {
	fn to_bytes(&self) -> Bytes {
		Bytes::from(self.to_vec())
	}

	fn from_bytes(bytes: Bytes) -> Result<Self, VersionedBytesError> {
		Self::from_arr(&bytes.as_ref())
	}
}

/**
 * Randomable
*/
impl<const DIM: usize> Randomable for VersionedBytes<DIM> {
	fn random() -> Self {
		let bytes: [u8; DIM] = arr::random(DIM).try_into().unwrap();
		VersionedBytes::new(0, bytes)
	}
}

/**
 * Equality
*/
impl<const DIM: usize> PartialEq for VersionedBytes<DIM> {
	fn eq(&self, other: &Self) -> bool {
		self.version == other.version && self.bytes == other.bytes
	}
}

impl<const DIM: usize> Eq for VersionedBytes<DIM> {}

/**
 * Impls
*/
impl<const DIM: usize> From<String> for VersionedBytes<DIM> {
	fn from(string: String) -> Self {
		VersionedBytes::from_string(&string).unwrap()
	}
}

impl<const DIM: usize> From<&str> for VersionedBytes<DIM> {
	fn from(string: &str) -> Self {
		VersionedBytes::from_string(string).unwrap()
	}
}

/**
 * Clone
*/
impl<const DIM: usize> Clone for VersionedBytes<DIM> {
	fn clone(&self) -> Self {
		VersionedBytes::new(self.version, self.bytes.clone())
	}
}

/**
 * Hash
*/
impl<const DIM: usize> Hash for VersionedBytes<DIM> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.bytes.hash(state);
	}
}

#[test]
fn test_versioned_bytes() {
	let vec256 = V256::random();
	let vec256_serialized = serialize(&vec256).unwrap();
	// assert_eq!(vec256_serialized.len(), 32 + 2);
	assert_eq!(vec256, deserialize(&vec256_serialized).unwrap());
}
