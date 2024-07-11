use wasm_bindgen::prelude::*;
//use js_sys::Uint8Array;
use crate::arr;
use crate::u256::*;
use getset::Getters;

pub type Hash = U256;

#[wasm_bindgen]
pub fn hash(data: &str) -> Hash {
	let arr: [u8; 32] = blake3::hash(data.as_ref()).into();
	U256::new(&arr).unwrap()
}

#[wasm_bindgen(js_name = verifyHash)]
pub fn verify_hash(data: &str, data_hash: Hash) -> bool {
	hash(data) == data_hash
}
