use base32::Alphabet;
use num_traits::PrimInt;
use rand::{Rng, RngCore};
//use std::convert::TryInto;
use crate::*;
use std::ops::BitXor;

/**
 * Errors
*/

#[derive(Debug)]
pub enum ArrError {
	InvalidBase32,
}

pub type Arr = [u8];
pub type Vrr = Vec<u8>;

/**
 * Conversions
*/

// string

pub fn to_string(arr: &Arr) -> String {
	String::from_utf8(arr.to_vec()).unwrap()
}

pub fn from_string(data: &String) -> Vrr {
	data.clone().into_bytes()
}

// to base32

pub fn to_base32(data: &[u8]) -> String {
	base32::encode(Alphabet::Rfc4648Lower { padding: false }, &data)
}

#[wasm_bindgen(js_name = "toBase32")]
pub fn _js_to_base32(data: Uint8Array) -> String {
	to_base32(&data.to_vec())
}

pub fn from_base32(data: &str) -> Result<Vrr, ArrError> {
	base32::decode(Alphabet::Rfc4648Lower { padding: false }, data)
		.ok_or(ArrError::InvalidBase32)
}

#[wasm_bindgen(js_name = "fromBase32")]
pub fn _js_from_base32(data: &str) -> Result<Uint8Array, JsValue> {
	from_base32(data)
		.map(|v| Uint8Array::from(v.as_slice()).into())
		.map_err(|e| JsValue::from_str(&format!("{:?}", e)))
}

pub fn is_valid_base32(data: &str) -> bool {
	from_base32(data).is_ok()
}

#[wasm_bindgen(js_name = "isValidBase32")]
pub fn _js_is_valid_base32(data: &str) -> bool {
	is_valid_base32(data)
}

/*
pub fn from_proto<M: prost::Message>(proto: &M) -> Vec<u8> {
	let mut buf = Vec::new();
	proto.encode(&mut buf).expect("Failed to encode proto");
	buf
}
*/

/*
 * Operations
*/
pub fn random(len: usize) -> Vrr {
	let mut rng = rand::thread_rng();
	let mut vec = vec![0u8; len];
	rng.fill(vec.as_mut_slice());
	vec
}

// Generate a random 32 byte id
type ArrId = [u8; 32];
pub fn id() -> ArrId {
	let mut id = [0u8; 32];
	rand::thread_rng().fill_bytes(&mut id);
	id
}

// Generic XOR function
pub fn xor<T, const N: usize>(a: &[T; N], b: &[T; N]) -> [T; N]
where
	T: PrimInt + BitXor<Output = T>, // Ensure T supports XOR and is a primitive integer
{
	let mut result = [T::zero(); N]; // Initialize array with zeros
	for i in 0..N {
		result[i] = a[i] ^ b[i];
	}
	result
}

// Generic Hamming distance function
pub fn hamming<T, const N: usize>(a: &[T; N], b: &[T; N]) -> u32
where
	T: PrimInt + BitXor<Output = T> + BitXor, // Ensure T supports XOR and is a primitive integer
{
	a.iter()
		.zip(b.iter())
		.map(|(&a, &b)| (a ^ b).count_ones())
		.sum()
}

// Generic Inverse Hamming distance function
pub fn inverse_hamming<T, const N: usize>(a: &[T; N], b: &[T; N]) -> u32
where
	T: PrimInt + BitXor<Output = T> + BitXor, // Ensure T supports XOR and is a primitive integer
{
	a.iter()
		.zip(b.iter())
		.map(|(&a, &b)| (a ^ b).count_zeros())
		.sum()
}

// Generic Andcount distance function
pub fn andcount<T, const N: usize>(a: &[T; N], b: &[T; N]) -> u32
where
	T: PrimInt + BitXor<Output = T> + BitXor, // Ensure T supports XOR and is a primitive integer
{
	a.iter()
		.zip(b.iter())
		.map(|(&a, &b)| (a & b).count_ones())
		.sum()
}

pub fn orcount<T, const N: usize>(a: &[T; N], b: &[T; N]) -> u32
where
	T: PrimInt + BitXor<Output = T> + BitXor, // Ensure T supports XOR and is a primitive integer
{
	a.iter()
		.zip(b.iter())
		.map(|(&a, &b)| (a | b).count_ones())
		.sum()
}

pub fn popcount<T, const N: usize>(a: &[T; N]) -> u32
where
	T: PrimInt + BitXor<Output = T> + BitXor, // Ensure T supports XOR and is a primitive integer
{
	a.iter().map(|&a| a.count_ones()).sum()
}

// jaccard distance
pub fn jaccard<T, const N: usize>(a: &[T; N], b: &[T; N]) -> f64
where
	T: PrimInt + BitXor<Output = T> + BitXor, // Ensure T supports XOR and is a primitive integer
{
	let intersection = andcount(a, b);
	let union = popcount(a) + popcount(b) - intersection;

	intersection as f64 / union as f64
}

// common prefix length
pub fn xor_leading_zeroes<T, const N: usize>(a: &[T; N], b: &[T; N]) -> u32
where
	T: PrimInt + BitXor<Output = T> + BitXor, // Ensure T supports XOR and is a primitive integer
{
	let mut count = 0;
	for i in 0..N {
		let xor = a[i] ^ b[i];
		if xor == T::zero() {
			count += 8;
		} else {
			count += xor.leading_zeros();
			break;
		}
	}
	count
}
