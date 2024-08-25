use crate::arr;
use crate::*;
use crate::*;

pub type Hash = V256;

pub fn hash(data: &[u8]) -> Hash {
	V256::new(
		0,
		&blake3::hash(data).as_bytes().to_vec(),
	)
}

#[wasm_bindgen]
pub fn _js_hash(data: Uint8Array) -> Hash {
	hash(&data.to_vec())
}

pub fn hash_verify(data: &[u8], data_hash: Hash) -> bool {
	hash(data) == data_hash
}

#[wasm_bindgen]
pub fn _js_hash_verify(data: Uint8Array, data_hash: Hash) -> bool {
	hash_verify(&data.to_vec(), data_hash)
}
