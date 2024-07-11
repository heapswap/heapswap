use wasm_bindgen::prelude::*;
//use js_sys::Uint8Array;
use super::u256::*;
use crate::arr;
use getset::Getters;

pub type Hash = U256;

#[wasm_bindgen]
pub fn hash(data: &str) -> Hash {
	let arr: [u8; 32] = blake3::hash(data.as_ref()).into();
	U256::new(&arr).unwrap()
}

#[wasm_bindgen]
pub fn verify(data: &str, data_hash: Hash) -> bool {
	hash(data) == data_hash
}
