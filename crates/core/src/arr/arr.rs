use base32::Alphabet;
use num_traits::PrimInt;
use rand::{Rng, RngCore};
use std::convert::TryInto;
use std::ops::BitXor;

/**
 * Errors
*/

pub enum ArrError {
	InvalidBase32,
}

pub type Arr = [u8];
pub type Vrr = Vec<u8>;

/**
 * Conversions
*/

pub fn to_string(arr: &Arr) -> String {
	String::from_utf8(arr.to_vec()).unwrap()
}

pub fn from_string(data: &String) -> Vrr {
	data.clone().into_bytes()
}

pub fn to_base32(data: &[u8]) -> String {
	base32::encode(Alphabet::Rfc4648Lower { padding: false }, &data)
}

pub fn from_base32(data: &str) -> Result<Vrr, ArrError> {
	base32::decode(Alphabet::Rfc4648Lower { padding: false }, data)
		.ok_or(ArrError::InvalidBase32)
}

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

pub fn concat(data: &[&Arr]) -> Vrr {
	data.iter().flat_map(|x| x.iter()).copied().collect()
}

// Generic XOR function for arrays of any size and integer type
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

// Generic Hamming distance function for arrays of any size and integer type
pub fn hamming<T, const N: usize>(a: &[T; N], b: &[T; N]) -> u32
where
	T: PrimInt + BitXor<Output = T> + BitXor, // Ensure T supports XOR and is a primitive integer
{
	a.iter()
		.zip(b.iter())
		.map(|(&a, &b)| (a ^ b).count_ones())
		.sum()
}
