use std::hash::Hash;
use crate::*;

pub type V96 = VersionedBytes;
pub type V256 = VersionedBytes;
pub type V512 = VersionedBytes;


#[derive(Debug, strum::Display)]
pub enum VersionedBytesError {
	InvalidBase32,
	InvalidVersion,
}

#[derive(Debug, Serialize, Deserialize, Getters)]
#[wasm_bindgen]
pub struct VersionedBytes {
	#[getset(get = "pub")]
	version: u16,
	#[getset(get = "pub")]
	#[serde(with = "serde_bytes")]
	bytes: Vec<u8>,
	#[serde(skip)]
	string: OnceCell<String>,
}

impl VersionedBytes {
	pub fn new(version: u16, bytes: &[u8]) -> Self {
		Self {
			version,
			bytes: bytes.to_vec(),
			string: OnceCell::new(),
		}
	}

	pub fn leading_zeros(&self) -> u32 {
		let mut count = 0;
		for i in 0..self.bytes.len() {
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
		for i in 0..self.bytes.len() {
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
	
	/**
	 * Random - workaround for wasm not supporting generics
	*/
	
	pub fn random96() -> Self {
		let bytes: Vec<u8> = arr::random(12).try_into().unwrap();
		VersionedBytes::new(0, bytes.as_slice())
	}

	pub fn random256() -> Self {
		let bytes: Vec<u8> = arr::random(32).try_into().unwrap();
		VersionedBytes::new(0, bytes.as_slice())
	}

	pub fn random512() -> Self {
		let bytes: Vec<u8> = arr::random(64).try_into().unwrap();
		VersionedBytes::new(0, bytes.as_slice())
	}
}

/**
 * Stringable
*/
impl Stringable<VersionedBytesError> for VersionedBytes {
	fn to_string(&self) -> String {
		self.string
			.get_or_init(|| arr::to_base32((&self.to_vec()).as_ref()))
			.clone()
	}

	fn from_string(s: &str) -> Result<Self, VersionedBytesError> {
		let vec = arr::from_base32(s)
			.map_err(|_| VersionedBytesError::InvalidBase32)?;
		VersionedBytes::from_arr(&vec)
	}
}
impl Into<String> for VersionedBytes {
	fn into(self) -> String {
		self.to_string()
	}
}

/**
 * Vecable
*/
impl Vecable<VersionedBytesError> for VersionedBytes {
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
impl Byteable<VersionedBytesError> for VersionedBytes {
	fn to_bytes(&self) -> Bytes {
		Bytes::from(self.to_vec())
	}

	fn from_bytes(bytes: Bytes) -> Result<Self, VersionedBytesError> {
		Self::from_arr(&bytes.as_ref())
	}
}

/**
 * Equality
*/
impl PartialEq for VersionedBytes {
	fn eq(&self, other: &Self) -> bool {
		self.version == other.version && self.bytes == other.bytes
	}
}

impl Eq for VersionedBytes {}

/**
 * Impls
*/
impl From<String> for VersionedBytes {
	fn from(string: String) -> Self {
		VersionedBytes::from_string(&string).unwrap()
	}
}

impl From<&str> for VersionedBytes {
	fn from(string: &str) -> Self {
		VersionedBytes::from_string(string).unwrap()
	}
}

/**
 * Clone
*/
impl Clone for VersionedBytes {
	fn clone(&self) -> Self {
		VersionedBytes::new(self.version, &self.bytes)
	}
}

/**
 * Hash
*/
impl Hash for VersionedBytes {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.bytes.hash(state);
	}
}

#[test]
fn test_versioned_bytes() {
	let vec256 = VersionedBytes::random256();
	let vec256_serialized = serialize(&vec256).unwrap();
	// assert_eq!(vec256_serialized.len(), 32 + 2);
	assert_eq!(vec256, deserialize(&vec256_serialized).unwrap());
}
