use crate::arr;
use crate::u256::*;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
//use getset::Getters;

pub type Hash = U256;

#[wasm_bindgen]
pub fn hash(data: &str) -> Hash {
	U256::hash(&data.as_bytes())
}

#[wasm_bindgen(js_name = verifyHash)]
pub fn verify_hash(data: &str, data_hash: Hash) -> bool {
	hash(data) == data_hash
}
