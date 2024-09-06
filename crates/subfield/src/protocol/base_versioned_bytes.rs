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

#[derive(Debug, Getters)]
#[wasm_bindgen]
pub struct VersionedBytes {
	#[getset(get = "pub")]
	version: VersionUsize,
	#[getset(get = "pub")]
	// #[serde(with = "serde_bytes")]
	data: Vec<u8>,
	// #[serde(skip)]
	string: OnceCell<String>,

	bigint: OnceCell<num_bigint::BigUint>,
}

impl VersionedBytes {
	/*
	Constructors
	*/
	pub fn new(version: VersionUsize, data: &[u8]) -> Self {
		Self {
			version,
			data: data.to_vec(),
			string: OnceCell::new(),
			bigint: OnceCell::new(),
		}
	}

	/*
	Getters
	*/
	pub fn bigint(&self) -> &num_bigint::BigUint {
		self.bigint.get_or_init(|| {
			num_bigint::BigUint::from_bytes_be(self.data.as_slice())
		})
	}

	/*
	Zeros
	*/
	pub fn zeros(version: VersionUsize, len: usize) -> Self {
		Self::new(version, &vec![0; len])
	}

	pub fn zeros_256(version: VersionUsize) -> Self {
		Self::zeros(version, 32)
	}

	pub fn zeros_512(version: VersionUsize) -> Self {
		Self::zeros(version, 64)
	}

	/*
	Ones
	*/
	pub fn ones(version: VersionUsize, len: usize) -> Self {
		Self::new(version, &vec![1; len])
	}

	pub fn ones_256(version: VersionUsize) -> Self {
		Self::ones(version, 32)
	}

	pub fn ones_512(version: VersionUsize) -> Self {
		Self::ones(version, 64)
	}

	/*
	Operations
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

	/*
	Random - workaround for wasm not supporting generics
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

	pub fn from_bigint(bigint: num_bigint::BigUint) -> Self {
		VersionedBytes::new(0, bigint.to_bytes_be().as_slice())
	}

	/*
	pub fn to_record_key(&self) -> libp2p::kad::RecordKey {
		libp2p::kad::RecordKey::from(self.data.as_slice().to_vec())
	}

	pub fn from_record_key(key: libp2p::kad::RecordKey) -> Self {
		VersionedBytes::new(0, key.to_vec().as_slice())
	}

	pub fn to_kbucket_key(&self) -> libp2p::kad::KBucketKey<Vec<u8>> {
		libp2p::kad::KBucketKey::new(self.data.as_slice().to_vec())
	}

	pub fn from_kbucket_key(key: libp2p::kad::KBucketKey<Vec<u8>>) -> Self {
		VersionedBytes::new(0, key.preimage().as_slice())
	}
	*/
}

/*
#[test]
fn test_versioned_bytes_to_key() {
	let key_vb = VersionedBytes::random256();
	let key_libp2p = key_vb.to_record_key();
	let key_back = VersionedBytes::from_record_key(key_libp2p);
	assert_eq!(key_vb, key_back);
}

#[test]
fn test_versioned_bytes_to_kbucket_key() {
	let key_vb = VersionedBytes::random256();
	let key_libp2p = key_vb.to_kbucket_key();
	let key_back = VersionedBytes::from_kbucket_key(key_libp2p);
	assert_eq!(key_vb, key_back);
}
*/

impl HasV256 for VersionedBytes {
	fn versioned_bytes(&self) -> &V256 {
		self
	}
}

/*
   Stringable
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

/*
   Vecable
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

/*
   Serde
*/
use serde::{Serialize, Serializer};

impl Serialize for VersionedBytes {
	fn serialize<S: Serializer>(
		&self,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(&self.to_vec())
	}
}

use serde::de::{self, Deserialize, Deserializer};

impl<'de> Deserialize<'de> for VersionedBytes {
	fn deserialize<D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<Self, D::Error> {
		let data = Vec::<u8>::deserialize(deserializer)?;
		VersionedBytes::from_arr(&data)
			.map_err(|_| de::Error::invalid_length(data.len(), &"valid bytes"))
	}
}

/*
   Equality
*/
impl PartialEq for VersionedBytes {
	fn eq(&self, other: &Self) -> bool {
		self.version == other.version && self.data == other.data
	}
}

impl Eq for VersionedBytes {}

/*
   Impls
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

/*
   Clone
*/
impl Clone for VersionedBytes {
	fn clone(&self) -> Self {
		VersionedBytes::new(self.version, &self.data)
	}
}

/*
   Hash
*/
impl Hash for VersionedBytes {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.to_vec().hash(state);
	}
}

/*
	Ord
*/
impl Ord for VersionedBytes {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.data().cmp(&other.data())
	}
}

impl PartialOrd for VersionedBytes {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.data().partial_cmp(other.data())
	}
}

#[wasm_bindgen]
impl VersionedBytes {
	/*
	Constructors
	*/

	#[wasm_bindgen(constructor)]
	pub fn _js_new(version: VersionUsize, data: Uint8Array) -> Self {
		VersionedBytes::new(version, data.to_vec().as_slice())
	}

	/*
	Getters
	*/
	#[wasm_bindgen(getter, js_name = "version")]
	pub fn _js_version(&self) -> VersionUsize {
		self.version().clone()
	}

	#[wasm_bindgen(getter, js_name = "data")]
	pub fn _js_data(&self) -> Uint8Array {
		self.data().clone().as_slice().into()
	}

	/*
	Random
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

	/*
	Byteable
	*/
	#[wasm_bindgen(js_name = "toBytes")]
	pub fn _js_to_bytes(&self) -> Uint8Array {
		self.to_vec().clone().as_slice().into()
	}

	#[wasm_bindgen(js_name = "fromBytes")]
	pub fn _js_from_bytes(data: Uint8Array) -> Self {
		VersionedBytes::from_arr(&data.to_vec()).unwrap()
	}

	/*
	Stringable
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
