use crate::*;
use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize, Getters)]
#[wasm_bindgen]
pub struct V256 {
	#[getset(get = "pub")]
	version: u16,
	#[getset(get = "pub")]
	#[serde(with = "serde_bytes")]
	bytes: [u8; 32],
	#[serde(skip)]
	string: OnceCell<String>,
}

impl V256 {
	pub fn new(version: u16, bytes: [u8; 32]) -> Self {
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
impl Randomable for V256 {
	fn random() -> Self {
		let bytes: [u8; 32] = arr::random(32).try_into().unwrap();
		Self::new(0, bytes)
	}
}

/**
 * Stringable
*/
impl Stringable<VersionedBytesError> for V256 {
	fn to_string(&self) -> String {
		self.string
			.get_or_init(|| arr::to_base32((&self.to_vec()).as_ref()))
			.clone()
	}

	fn from_string(s: &str) -> Result<Self, VersionedBytesError> {
		let vec = arr::from_base32(s)
			.map_err(|_| VersionedBytesError::InvalidBase32)?;
		let arr: [u8; 32] = vec
			.as_slice()
			.try_into()
			.map_err(|_| VersionedBytesError::InvalidVersion)?;
		V256::from_arr(&arr)
	}
}
impl Into<String> for V256 {
	fn into(self) -> String {
		self.to_string()
	}
}

/**
 * Vecable
*/
impl Vecable<VersionedBytesError> for V256 {
	fn from_arr(arr: &[u8]) -> Result<Self, VersionedBytesError> {
		let (bytes, version) = arr.split_at(arr.len() - 2);
		let version = u16::from_le_bytes(version.try_into().unwrap());
		Ok(V256::new(version, bytes.try_into().unwrap()))
	}

	fn to_vec(&self) -> Vec<u8> {
		let version_bytes: [u8; 2] = self.version.to_le_bytes();
		[self.bytes.as_slice(), &version_bytes].concat()
	}
}

/**
 * Byteable
*/
impl Byteable<VersionedBytesError> for V256 {
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
impl PartialEq for V256 {
	fn eq(&self, other: &Self) -> bool {
		self.version() == other.version() && self.bytes() == other.bytes()
	}
}
impl Eq for V256 {}

/**
 * Impls
*/
impl From<String> for V256 {
	fn from(string: String) -> Self {
		Self::from_string(&string).unwrap()
	}
}

impl From<&str> for V256 {
	fn from(string: &str) -> Self {
		V256::from_string(string).unwrap()
	}
}

/**
 * Clone
*/
impl Clone for V256 {
	fn clone(&self) -> Self {
		V256::new(self.version, self.bytes.clone())
	}
}

/**
 *  * Hash
 * */
impl Hash for V256 {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.bytes.hash(state);
	}
}

#[test]
fn test_v256() {
	let vec256 = V256::random();
	let vec256_serialized = serialize(&vec256).unwrap();
	// assert_eq!(vec256_serialized.len(), 32 + 2);
	assert!(vec256 == deserialize::<V256>(&vec256_serialized).unwrap());
}


fn validate_bytes(bytes: Uint8Array) -> [u8; 32] {
	bytes.to_vec().try_into().unwrap()
}


#[wasm_bindgen]
impl V256 {
	#[wasm_bindgen(constructor)]
	pub fn _js_new(version: u16, bytes: Uint8Array) -> Self {
		Self::new(version, validate_bytes(bytes))
	}
	
	#[wasm_bindgen(getter, js_name = "version")]
	pub fn _js_version(&self) -> u16 {
		self.version
	}
	
	#[wasm_bindgen(getter, js_name = "bytes")]
	pub fn _js_bytes(&self) -> Uint8Array {
		Uint8Array::from(self.bytes.to_vec())
	}
}