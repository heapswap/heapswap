use crate::arr;
use crate::*;
use getset::Getters;
use js_sys::Uint8Array;
use once_cell::sync::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, strum::Display)]
pub enum U256Error {
	UnableToSerialize,
	UnableToDeserialize,
	InvalidBase32,
	InvalidLength,
}

const UNPACKED_LENGTH: usize = 32;
const PACKED_LENGTH: usize = 4;
const PACKED_CHUNKS: usize = UNPACKED_LENGTH / PACKED_LENGTH;

#[wasm_bindgen]
#[derive(Getters, Debug, Clone)]
pub struct U256 {
	unpacked: OnceCell<[u8; UNPACKED_LENGTH]>,
	packed: OnceCell<[u64; PACKED_LENGTH]>,
	popcount: OnceCell<u32>,
	string: OnceCell<String>,
}

impl Serialize for U256 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let string_repr = self.to_string();
		serializer.serialize_str(&string_repr)
	}
}

impl<'de> Deserialize<'de> for U256 {
	fn deserialize<D>(deserializer: D) -> Result<U256, D::Error>
	where
		D: Deserializer<'de>,
	{
		let string_repr = String::deserialize(deserializer)
			.map_err(serde::de::Error::custom)?;
		U256::from_string(&string_repr).map_err(serde::de::Error::custom)
	}
}

impl U256 {
	/**
	 * Hashing
	 */
	pub fn hash(data: &[u8]) -> U256 {
		let _hash: [u8; 32] = blake3::hash(data).into();
		U256::new(_hash)
	}

	pub fn verify(data: &[u8], data_hash: U256) -> bool {
		U256::hash(data) == data_hash
	}
}

impl U256 {
	/**
	 * Constructors
		*/

	pub fn new(unpacked: [u8; UNPACKED_LENGTH]) -> U256 {
		U256 {
			unpacked: OnceCell::from(unpacked),
			packed: OnceCell::new(),
			popcount: OnceCell::new(),
			string: OnceCell::new(),
		}
	}

	pub fn new_from_packed(packed: &[u64]) -> U256 {
		let packed: [u64; PACKED_LENGTH] = packed
			.try_into()
			.map_err(|_| U256Error::InvalidLength)
			.unwrap();

		let unpacked = U256::unpack(&packed);

		U256 {
			unpacked: OnceCell::from(unpacked),
			packed: OnceCell::from(packed),
			popcount: OnceCell::new(),
			string: OnceCell::new(),
		}
	}

	/**
	 * Getters
		*/

	fn unpack(packed: &[u64; PACKED_LENGTH]) -> [u8; UNPACKED_LENGTH] {
		let mut unpacked = [0u8; UNPACKED_LENGTH];
		packed.iter().enumerate().for_each(|(i, chunk)| {
			unpacked[i * 8..(i + 1) * 8].copy_from_slice(&chunk.to_le_bytes());
		});
		unpacked
	}

	fn pack(unpacked: &[u8; UNPACKED_LENGTH]) -> [u64; PACKED_LENGTH] {
		let mut packed = [0u64; PACKED_LENGTH];

		unpacked
			.chunks(PACKED_CHUNKS)
			.enumerate()
			.for_each(|(i, chunk)| {
				packed[i] = u64::from_le_bytes(chunk.try_into().unwrap());
			});

		packed
	}

	pub fn unpacked(&self) -> &[u8; UNPACKED_LENGTH] {
		self.unpacked.get_or_init(|| U256::unpack(self.packed()))
	}

	pub fn packed(&self) -> &[u64; PACKED_LENGTH] {
		self.packed.get_or_init(|| U256::pack(self.unpacked()))
	}

	pub fn popcount(&self) -> &u32 {
		self.popcount.get_or_init(|| arr::popcount(self.packed()))
	}

	/**
	 * Byteable
		*/
	pub fn to_bytes(&self) -> Vec<u8> {
		self.unpacked().to_vec()
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<U256, U256Error> {
		let bytes: [u8; UNPACKED_LENGTH] =
			bytes.try_into().map_err(|_| U256Error::InvalidLength)?;
		Ok(U256::new(bytes))
	}
}

#[wasm_bindgen]
impl U256 {
	/**
	 * Constructors
		*/
	#[wasm_bindgen(constructor)]
	pub fn _js_new(unpacked: Uint8Array) -> Result<U256, U256Error> {
		let unpacked: [u8; UNPACKED_LENGTH] = unpacked
			.to_vec()
			.as_slice()
			.try_into()
			.map_err(|_| U256Error::InvalidLength)
			.unwrap();
		Ok(U256::new(unpacked))
	}

	#[wasm_bindgen]
	pub fn random() -> U256 {
		let unpacked: [u8; UNPACKED_LENGTH] =
			arr::random(UNPACKED_LENGTH).try_into().unwrap();
		U256::new(unpacked)
	}

	/**
	 * Operations
		*/
	#[wasm_bindgen]
	pub fn jaccard(&self, other: &U256) -> f64 {
		let intersection = arr::andcount(self.packed(), other.packed());
		let union = self.popcount() + other.popcount() - intersection;
		intersection as f64 / union as f64
	}

	#[wasm_bindgen]
	pub fn xor(&self, other: &U256) -> U256 {
		//U256::new_from_packed(&arr::xor(self.packed(), other.packed()))
		U256::new(arr::xor(self.unpacked(), other.unpacked()))
	}

	#[wasm_bindgen]
	pub fn hamming(&self, other: &U256) -> u32 {
		arr::hamming(self.packed(), other.packed())
	}

	#[wasm_bindgen]
	pub fn common_prefix(&self, other: &U256) -> u32 {
		arr::common_prefix(self.unpacked(), other.unpacked())
	}

	#[wasm_bindgen]
	pub fn equals(&self, other: &U256) -> bool {
		self == other
	}

	/**
	 * Byteable
		*/

	#[wasm_bindgen(js_name = toBytes)]
	pub fn _js_to_bytes(&self) -> Uint8Array {
		Uint8Array::from(self.unpacked().to_vec().as_slice())
	}

	#[wasm_bindgen(js_name = fromBytes)]
	pub fn _js_from_bytes(bytes: &Uint8Array) -> Result<U256, U256Error> {
		let unpacked: [u8; UNPACKED_LENGTH] = bytes
			.to_vec()
			.as_slice()
			.try_into()
			.map_err(|_| U256Error::InvalidLength)?;
		Ok(U256::new(unpacked))
	}

	/**
	 * Stringable
		*/
	#[wasm_bindgen(js_name = toString)]
	pub fn to_string(&self) -> String {
		self.string
			.get_or_init(|| arr::to_base32(self.unpacked()))
			.clone()
	}

	#[wasm_bindgen(js_name = fromString)]
	pub fn from_string(string: &str) -> Result<U256, U256Error> {
		let unpacked: [u8; 32] = arr::from_base32(string)
			.map_err(|_| U256Error::InvalidBase32)?
			.try_into()
			.map_err(|_| U256Error::InvalidLength)?;

		Ok(U256 {
			unpacked: OnceCell::from(unpacked),
			packed: OnceCell::new(),
			popcount: OnceCell::new(),
			string: OnceCell::from(string.to_string()),
		})
	}

	/*
	 * Hashing
		*/

	#[wasm_bindgen(js_name = hash)]
	pub fn _js_hash(data: Uint8Array) -> U256 {
		U256::hash(data.to_vec().as_slice())
	}

	#[wasm_bindgen(js_name = verifyHash)]
	pub fn _js_verify(data: Uint8Array, data_hash: U256) -> bool {
		U256::verify(data.to_vec().as_slice(), data_hash)
	}
}

impl PartialEq for U256 {
	fn eq(&self, other: &Self) -> bool {
		self.unpacked() == other.unpacked()
	}
}
