use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

/**
 * Conversions
*/
pub fn to_base32(data: &[u8]) -> String {
	base32::encode(
		base32::Alphabet::Rfc4648Lower { padding: false },
		&data.to_vec(),
	)
}

pub fn from_base32(data: &str) -> Uint8Array {
	let bytes =
		base32::decode(base32::Alphabet::Rfc4648Lower { padding: false }, data)
			.unwrap();
	Uint8Array::from(bytes.as_slice())
}
