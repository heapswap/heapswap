use crate::*;
use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize, Getters)]
#[wasm_bindgen]
pub struct V512 {
	#[getset(get = "pub")]
	version: u16,
	#[getset(get = "pub")]
	#[serde(with = "serde_bytes")]
	bytes: [u8; 64],
	#[serde(skip)]
	string: OnceCell<String>,
}

impl V512 {
	pub fn new(version: u16, bytes: [u8; 64]) -> Self {
		Self {
			version,
			bytes,
			string: OnceCell::new(),
		}
	}
}

/**
 * Randomable
*/
impl Randomable for V512 {
	fn random() -> Self {
		let bytes: [u8; 64] = arr::random(64).try_into().unwrap();
		Self::new(0, bytes)
	}
}

/**
 * Stringable
*/
impl Stringable<VersionedBytesError> for V512 {
	fn to_string(&self) -> String {
		self.string
			.get_or_init(|| arr::to_base32((&self.to_vec()).as_ref()))
			.clone()
	}

	fn from_string(s: &str) -> Result<Self, VersionedBytesError> {
		let vec = arr::from_base32(s)
			.map_err(|_| VersionedBytesError::InvalidBase32)?;
		let arr: [u8; 64] = vec
			.as_slice()
			.try_into()
			.map_err(|_| VersionedBytesError::InvalidVersion)?;
		V512::from_arr(&arr)
	}
}
impl Into<String> for V512 {
	fn into(self) -> String {
		self.to_string()
	}
}

/**
 * Vecable
*/
impl Vecable<VersionedBytesError> for V512 {
	fn from_arr(arr: &[u8]) -> Result<Self, VersionedBytesError> {
		let (bytes, version) = arr.split_at(arr.len() - 2);
		let version = u16::from_le_bytes(version.try_into().unwrap());
		Ok(V512::new(version, bytes.try_into().unwrap()))
	}

	fn to_vec(&self) -> Vec<u8> {
		let version_bytes: [u8; 2] = self.version.to_le_bytes();
		[self.bytes.as_slice(), &version_bytes].concat()
	}
}

/**
 * Byteable
*/
impl Byteable<VersionedBytesError> for V512 {
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
impl PartialEq for V512 {
	fn eq(&self, other: &Self) -> bool {
		self.version() == other.version() && self.bytes() == other.bytes()
	}
}
impl Eq for V512 {}

/**
 * Impls
*/
impl From<String> for V512 {
	fn from(string: String) -> Self {
		Self::from_string(&string).unwrap()
	}
}

impl From<&str> for V512 {
	fn from(string: &str) -> Self {
		V512::from_string(string).unwrap()
	}
}

/**
 * Clone
*/
impl Clone for V512 {
	fn clone(&self) -> Self {
		V512::new(self.version, self.bytes.clone())
	}
}

/**
 *  * Hash
 * */
impl Hash for V512 {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.bytes.hash(state);
	}
}

#[test]
fn test_v512() {
	let vec512 = V512::random();
	let vec512_serialized = serialize(&vec512).unwrap();
	// assert_eq!(vec512_serialized.len(), 64 + 2);
	assert!(vec512 == deserialize::<V512>(&vec512_serialized).unwrap());
}
