use crate::*;
use std::hash::Hash;

pub type V96 = VersionedBytes;
pub type V256 = VersionedBytes;
pub type V512 = VersionedBytes;

#[derive(Debug, strum::Display)]
pub enum VersionedBytesError {
	InvalidBase32,
	InvalidVersion,
}

type VersionUsize = u32;
const VERSION_BYTES: usize = 4;

#[derive(Debug, Serialize, Deserialize, Getters)]
#[wasm_bindgen]
pub struct VersionedBytes {
	#[getset(get = "pub")]
	version: VersionUsize,
	#[getset(get = "pub")]
	#[serde(with = "serde_bytes")]
	data: Vec<u8>,
	#[serde(skip)]
	string: OnceCell<String>,
}

impl VersionedBytes {
	
	/**
	 * Constructors
	*/
	pub fn new(version: VersionUsize, data: &[u8]) -> Self {
		Self {
			version,
			data: data.to_vec(),
			string: OnceCell::new(),
		}
	}

	pub fn zero(version: VersionUsize, len: usize) -> Self {
		Self::new(version, &vec![0; len])
	}
	
	pub fn one(version: VersionUsize, len: usize) -> Self {
		Self::new(version, &vec![1; len])
	}
	
	/**
	 * Operations
	*/
	pub fn leading_zeros(&self) -> u32 {
		let mut count = 0;
		for i in 0..self.data.len() {
			if self.data[i] == 0 {
				count += 8;
			} else {
				count += self.data[i].leading_zeros();
				break;
			}
		}
		count
	}

	pub fn xor_leading_zeros(&self, other: &Self) -> u32 {
		let mut count = 0;
		for i in 0..self.data.len() {
			let xor = self.data[i] ^ other.data[i];
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
		let data: Vec<u8> = arr::random(12).try_into().unwrap();
		VersionedBytes::new(0, data.as_slice())
	}

	pub fn random256() -> Self {
		let data: Vec<u8> = arr::random(32).try_into().unwrap();
		VersionedBytes::new(0, data.as_slice())
	}

	pub fn random512() -> Self {
		let data: Vec<u8> = arr::random(64).try_into().unwrap();
		VersionedBytes::new(0, data.as_slice())
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
		let (data, version) = arr.split_at(arr.len() - VERSION_BYTES);
		let version = VersionUsize::from_le_bytes(version.try_into().unwrap());
		Ok(VersionedBytes::new(version, data.try_into().unwrap()))
	}

	fn to_vec(&self) -> Vec<u8> {
		let version_data: [u8; VERSION_BYTES] = self.version.to_le_bytes();
		[self.data.as_slice(), &version_data].concat()
	}
}

/**
 * Protoable
*/
impl Protoable<subfield_proto::VersionedBytes, VersionedBytesError> for VersionedBytes {
	fn from_proto(proto: subfield_proto::VersionedBytes) -> Result<Self, VersionedBytesError> {
		Ok(VersionedBytes::new(proto.version, proto.data.as_slice()))
	}

	fn to_proto(&self) -> Result<subfield_proto::VersionedBytes, VersionedBytesError> {
		Ok(subfield_proto::VersionedBytes {
			version: self.version,
			data: self.data.clone().into(),
		})
	}
	
	fn from_proto_bytes(bytes: Bytes) -> Result<Self, VersionedBytesError> {
		Ok(Self::from_proto(proto::deserialize::<subfield_proto::VersionedBytes>(bytes).unwrap())?)
	}
	
	fn to_proto_bytes(&self) -> Result<Bytes, VersionedBytesError> {
		Ok(proto::serialize::<subfield_proto::VersionedBytes>(self.to_proto()?).unwrap())
	}
}



/**
 * Byteable
*/
/*
impl Byteable<VersionedBytesError> for VersionedBytes {
	fn to_bytes(&self) -> Bytes {
		Bytes::from(self.to_vec())
	}

	fn from_bytes(bytes: Bytes) -> Result<Self, VersionedBytesError> {
		Self::from_arr(&bytes.as_ref())
	}
}
*/

/**
 * Equality
*/
impl PartialEq for VersionedBytes {
	fn eq(&self, other: &Self) -> bool {
		self.version == other.version && self.data == other.data
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
		VersionedBytes::new(self.version, &self.data)
	}
}

/**
 * Hash
*/
impl Hash for VersionedBytes {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.to_vec().hash(state);
	}
}

#[wasm_bindgen]
impl VersionedBytes {
	/**
	 * Constructors
		*/

	#[wasm_bindgen(constructor)]
	pub fn _js_new(version: VersionUsize, data: Uint8Array) -> Self {
		VersionedBytes::new(version, data.to_vec().as_slice())
	}

	/**
	 * Getters
		*/
	#[wasm_bindgen(getter, js_name = "version")]
	pub fn _js_version(&self) -> VersionUsize {
		self.version().clone()
	}

	#[wasm_bindgen(getter, js_name = "data")]
	pub fn _js_data(&self) -> Uint8Array {
		self.data().clone().as_slice().into()
	}

	/**
	 * Random
		*/

	#[wasm_bindgen(js_name = "random96")]
	pub fn _js_random96() -> Self {
		VersionedBytes::random96()
	}

	#[wasm_bindgen(js_name = "random256")]
	pub fn _js_random256() -> Self {
		VersionedBytes::random256()
	}

	#[wasm_bindgen(js_name = "random512")]
	pub fn _js_random512() -> Self {
		VersionedBytes::random512()
	}

	/**
	 * Byteable
		*/
	#[wasm_bindgen(js_name = "toBytes")]
	pub fn _js_to_bytes(&self) -> Uint8Array {
		self.to_vec().clone().as_slice().into()
	}

	#[wasm_bindgen(js_name = "fromBytes")]
	pub fn _js_from_bytes(data: Uint8Array) -> Self {
		VersionedBytes::from_arr(&data.to_vec()).unwrap()
	}

	/**
	 * Stringable
		*/
	#[wasm_bindgen(js_name = "toString")]
	pub fn _js_to_string(&self) -> String {
		self.to_string()
	}

	#[wasm_bindgen(js_name = "fromString")]
	pub fn _js_from_string(string: &str) -> Self {
		VersionedBytes::from_string(&string).unwrap()
	}
}
