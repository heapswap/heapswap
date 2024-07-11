use crate::arr;
use crate::*;
use getset::Getters;
use js_sys::Uint8Array;
use once_cell::sync::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub enum U256Error {
	InvalidBase32,
	InvalidLength,
}

const UNPACKED_LENGTH: usize = 32;
const PACKED_LENGTH: usize = 4;

#[wasm_bindgen]
#[derive(Getters, Debug, Clone)]
pub struct U256 {
	#[getset(get = "pub")]
	unpacked: [u8; UNPACKED_LENGTH],
	packed: OnceCell<[u64; PACKED_LENGTH]>,
	popcount: OnceCell<u32>,
}

#[wasm_bindgen]
impl U256 {
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

	/**
	 * Getters
		*/
	fn pack(&self) -> [u64; PACKED_LENGTH] {
		let mut packed = [0u64; PACKED_LENGTH];
		for (i, chunk) in self.unpacked().chunks_exact(8).enumerate() {
			packed[i] = u64::from_le_bytes(chunk.try_into().unwrap());
		}
		packed
	}

	fn packed(&self) -> &[u64; 4] {
		self.packed.get_or_init(|| self.pack())
	}

	fn popcount(&self) -> &u32 {
		self.popcount.get_or_init(|| arr::popcount(self.packed()))
	}

	/**
	 * Operations
		*/
	#[wasm_bindgen]
	pub fn jaccard(&self, other: &U256) -> f64 {
		let intersection = arr::andcount(self.packed(), other.packed());
		//let union = arr::orcount(self.packed(), other.packed());
		let union = self.popcount() + other.popcount() - intersection;

		intersection as f64 / union as f64
	}

	#[wasm_bindgen]
	pub fn random() -> U256 {
		let unpacked = arr::id();
		U256::new(unpacked.as_slice()).unwrap()
	}

	/**
	 * Byteable
		*/
	#[wasm_bindgen]
	pub fn toBytes(&self) -> Uint8Array {
		Uint8Array::from(self.unpacked().to_vec().as_slice())
	}

	#[wasm_bindgen]
	pub fn fromBytes(bytes: &Uint8Array) -> Result<U256, U256Error> {
		let unpacked = bytes.to_vec();
		U256::new(unpacked.as_slice())
	}

	/**
	 * Stringable
		*/
	#[wasm_bindgen]
	pub fn toString(&self) -> String {
		arr::to_base32(self.unpacked())
	}

	#[wasm_bindgen]
	pub fn fromString(string: &str) -> Result<U256, U256Error> {
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
