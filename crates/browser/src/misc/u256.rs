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
const EXTENDED_UNPACKED_LENGTH: usize = 2 * UNPACKED_LENGTH;
const EXTENDED_PACKED_LENGTH: usize = 2 * PACKED_LENGTH;
const EXTENDED_PACKED_CHUNKS: usize =
	EXTENDED_UNPACKED_LENGTH / EXTENDED_PACKED_LENGTH;

#[wasm_bindgen]
#[derive(Getters, Debug, Clone)]
pub struct U256 {
	#[getset(get = "pub")]
	unpacked: [u8; UNPACKED_LENGTH],
	packed: OnceCell<[u64; EXTENDED_PACKED_LENGTH]>,
	popcount: OnceCell<u32>,
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

#[wasm_bindgen]
impl U256 {
	/**
	 * Constructors
		*/

	pub fn new(unpacked: &[u8]) -> Result<U256, U256Error> {
		let unpacked = unpacked
			.try_into()
			.map_err(|_| U256Error::InvalidLength)
			.unwrap();
		Ok(U256 {
			unpacked,
			packed: OnceCell::new(),
			popcount: OnceCell::new(),
		})
	}

	#[wasm_bindgen(js_name = new, constructor)]
	pub fn from_uint8array(unpacked: Uint8Array) -> Result<U256, U256Error> {
		let unpacked = unpacked.to_vec();
		U256::new(unpacked.as_slice())
	}

	#[wasm_bindgen]
	pub fn random() -> U256 {
		let unpacked = arr::random(UNPACKED_LENGTH);
		U256::new(unpacked.as_slice()).unwrap()
	}

	/**
	 * Getters
		*/

	// packing converts the u8 array into a u64 array, for faster bitwise operations
	// it also extends the 256 bit key to 512 bits by appending the hash of the key
	// hopefully this extension should be enough to prevent jaccard collisions
	fn pack(&self) -> [u64; EXTENDED_PACKED_LENGTH] {
		let mut packed = [0u64; EXTENDED_PACKED_LENGTH];

		let extension: [u8; UNPACKED_LENGTH] =
			blake3::hash(self.unpacked()).into();
		let extended: [u8; EXTENDED_UNPACKED_LENGTH] =
			arr::concat(&[&self.unpacked, &extension])
				.try_into()
				.unwrap();

		for (i, chunk) in
			extended.chunks_exact(EXTENDED_PACKED_CHUNKS).enumerate()
		{
			packed[i] = u64::from_le_bytes(chunk.try_into().unwrap());
		}

		packed
	}

	fn packed(&self) -> &[u64; EXTENDED_PACKED_LENGTH] {
		self.packed.get_or_init(|| self.pack())
	}

	// cache the popcount
	// note that this operates on packed, to get the counts of the extended key
	fn popcount(&self) -> &u32 {
		self.popcount.get_or_init(|| arr::popcount(self.packed()))
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
	pub fn equals(&self, other: &U256) -> bool {
		self == other
	}

	/**
	 * Byteable
		*/
	#[wasm_bindgen(js_name = toBytes)]
	pub fn to_bytes(&self) -> Uint8Array {
		Uint8Array::from(self.unpacked().to_vec().as_slice())
	}

	#[wasm_bindgen(js_name = fromBytes)]
	pub fn from_bytes(bytes: &Uint8Array) -> Result<U256, U256Error> {
		let unpacked = bytes.to_vec();
		U256::new(unpacked.as_slice())
	}

	/**
	 * Stringable
		*/
	#[wasm_bindgen(js_name = toString)]
	pub fn to_string(&self) -> String {
		arr::to_base32(self.unpacked())
	}

	#[wasm_bindgen(js_name = fromString)]
	pub fn from_string(string: &str) -> Result<U256, U256Error> {
		let unpacked =
			arr::from_base32(string).map_err(|_| U256Error::InvalidBase32)?;
		U256::new(unpacked.as_slice())
	}
}

impl PartialEq for U256 {
	fn eq(&self, other: &Self) -> bool {
		self.unpacked() == other.unpacked()
	}
}
