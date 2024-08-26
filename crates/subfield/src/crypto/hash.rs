use crate::arr;
use crate::*;
use js_sys::{JsString, Uint8Array};
use wasm_bindgen::prelude::*;

pub type Hash = V256;

// hash

pub fn hash(data: &[u8]) -> Hash {
	V256::new(0, &blake3::hash(data).as_bytes().to_vec())
}

#[wasm_bindgen(js_name = "hash")]
pub fn _js_hash(data: JsValue) -> Result<Hash, JsValue> {
	if let Some(uint8_array) = data.dyn_ref::<Uint8Array>() {
		Ok(hash(&uint8_array.to_vec()))
	} else if let Some(js_string) = data.as_string() {
		Ok(hash(js_string.as_bytes()))
	} else {
		Err(JsValue::from_str("Invalid input type"))
	}
}

// hash verify

pub fn hash_verify(data: &[u8], data_hash: Hash) -> bool {
	hash(data) == data_hash
}

#[wasm_bindgen(js_name = "hashVerify")]
pub fn _js_hash_verify(
	data: JsValue,
	data_hash: Hash,
) -> Result<bool, JsValue> {
	if let Some(uint8_array) = data.dyn_ref::<Uint8Array>() {
		Ok(hash_verify(&uint8_array.to_vec(), data_hash))
	} else if let Some(js_string) = data.as_string() {
		Ok(hash_verify(js_string.as_bytes(), data_hash))
	} else {
		Err(JsValue::from_str("Invalid input type"))
	}
}
