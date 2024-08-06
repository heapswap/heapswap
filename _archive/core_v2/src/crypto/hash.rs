use crate::arr;
use crate::vector::*;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
//use getset::Getters;

pub type Hash = U256;

pub fn hash(data: &[u8]) -> Hash {
	U256::hash(&data)
}

pub fn verify_hash(data: &[u8], data_hash: Hash) -> bool {
	hash(data) == data_hash
}

#[wasm_bindgen(js_name = hash)]
pub fn _js_hash(data: &str) -> Hash {
	U256::hash(&data.as_bytes())
}

#[wasm_bindgen(js_name = verifyHash)]
pub fn _js_verify_hash(data: &str, data_hash: Hash) -> bool {
	_js_hash(data) == data_hash
}
