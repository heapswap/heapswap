use crate::{arr, arr::*, bys, constants::NS, traits::*};
use bytes::Bytes;
use derive_getters::Getters;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use rand::RngCore;
use std::fmt;
use std::{cmp::Ordering, ops::BitXor};
use std::{collections::HashMap, sync::Mutex};
//use heapswap_protos::u256::U256 as U256Proto;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, strum::Display, PartialEq)]
pub enum U256Error {
	InvalidBase32,
	InvalidLength,
}

pub type Packed256 = [u64; 4];
pub type Unpacked256 = [u8; 32];

#[derive(Debug, Clone, Getters)]
pub struct U256 {
	packed: Packed256,
	unpacked: OnceCell<Unpacked256>,
}

/**
 * Serialization - display as string
*/
impl Serialize for U256 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let string_repr = self.to_string();
		serializer.serialize_str(&string_repr)
	}
}

impl<'de> Deserialize<'de> for U256 {
	fn deserialize<D>(deserializer: D) -> Result<U256, D::Error>
	where
		D: Deserializer<'de>,
	{
		let string_repr = String::deserialize(deserializer)?;
		U256::from_string(&string_repr).map_err(serde::de::Error::custom)
	}
}

/**
 * Packing
*/
fn unpack(packed: Packed256) -> Unpacked256 {
	let mut unpacked = [0u8; 32];
	unpacked[0..8].copy_from_slice(&packed[0].to_le_bytes());
	unpacked[8..16].copy_from_slice(&packed[1].to_le_bytes());
	unpacked[16..24].copy_from_slice(&packed[2].to_le_bytes());
	unpacked[24..32].copy_from_slice(&packed[3].to_le_bytes());
	unpacked
}

fn pack(unpacked: &Unpacked256) -> Packed256 {
	[
		u64::from_le_bytes(unpacked[0..8].try_into().unwrap()),
		u64::from_le_bytes(unpacked[8..16].try_into().unwrap()),
		u64::from_le_bytes(unpacked[16..24].try_into().unwrap()),
		u64::from_le_bytes(unpacked[24..32].try_into().unwrap()),
	]
}

impl U256 {
	pub fn new(packed: Packed256) -> Self {
		U256::from_packed(packed)
	}

	pub fn from_unpacked(unpacked: Unpacked256) -> Self {
		U256 {
			packed: pack(&unpacked),
			unpacked: OnceCell::from(unpacked),
		}
	}

	pub fn from_packed(packed: Packed256) -> Self {
		U256 {
			packed: packed,
			unpacked: OnceCell::from(unpack(packed)),
		}
	}

	pub fn xor(&self, other: &U256) -> U256 {
		U256::from_packed(xor(self.packed(), other.packed()))
	}

	pub fn hamming(&self, other: &U256) -> u32 {
		hamming(self.packed(), other.packed())
	}

	pub fn zero() -> Self {
		U256::new([0, 0, 0, 0])
	}
}

impl Default for U256 {
	fn default() -> Self {
		U256::zero()
	}
}

/**
 * Ordering  
*/
/*
impl PartialOrd for U256 {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		if self.packed[0] != other.packed[0] {
			Some(self.packed[0].cmp(&other.packed[0]))
		} else if self.packed[1] != other.packed[1] {
			Some(self.packed[1].cmp(&other.packed[1]))
		} else if self.packed[2] != other.packed[2] {
			Some(self.packed[2].cmp(&other.packed[2]))
		} else {
			Some(self.packed[3].cmp(&other.packed[3]))
		}
	}
}
*/
impl PartialOrd for U256 {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.packed
			.iter()
			.zip(other.packed.iter())
			.find_map(|(a, b)| if a != b { Some(a.cmp(b)) } else { None })
			.or(Some(Ordering::Equal))
	}
}

impl Ord for U256 {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(other).unwrap()
	}
}

impl PartialEq for U256 {
	fn eq(&self, other: &Self) -> bool {
		self.packed == other.packed
	}
}

impl Eq for U256 {}

/**
 * Xor
*/

impl BitXor for U256 {
	type Output = Self;

	fn bitxor(self, rhs: Self) -> Self {
		U256::from_packed(xor(&self.packed(), &rhs.packed()))
	}
}

/**
 * Conversions
*/

impl Randomable<U256Error> for U256 {
	fn random() -> Result<Self, U256Error> {
		let mut unpacked = [0u8; 32];
		rand::rngs::OsRng.fill_bytes(&mut unpacked);
		U256::from_arr(&unpacked)
	}
}

impl Arrable<Unpacked256, U256Error> for U256 {
	fn to_arr(&self) -> Unpacked256 {
		self.unpacked.get_or_init(|| unpack(self.packed)).clone()
	}

	fn from_arr(unpacked: &Unpacked256) -> Result<Self, U256Error> {
		Ok(U256 {
			packed: pack(unpacked),
			unpacked: OnceCell::from(unpacked.clone()),
		})
	}
}

impl Byteable<U256Error> for U256 {
	fn to_bytes(&self) -> Bytes {
		Bytes::copy_from_slice(&self.to_arr())
	}

	fn from_bytes(bytes: &Bytes) -> Result<Self, U256Error> {
		if bytes.len() != 32 {
			return Err(U256Error::InvalidLength);
		}
		Ok(U256::from_arr(&bytes[..].try_into().unwrap()).unwrap())
	}
}

impl Stringable<U256Error> for U256 {
	fn to_string(&self) -> String {
		//bys::to_base32(&self.to_bytes())
		arr::to_base32(&self.to_arr())
	}

	fn from_string(string: &str) -> Result<Self, U256Error> {
		//let bytes =
		//bys::from_base32(string).map_err(|_| U256Error::InvalidLength)?;

		//U256::from_bytes(&bytes)

		let vec =
			arr::from_base32(string).map_err(|_| U256Error::InvalidLength)?;

		if vec.len() == 32 {
			let mut arr = [0u8; 32];
			arr.copy_from_slice(&vec);
			Ok(U256::from_arr(&arr)?)
		} else {
			Err(U256Error::InvalidLength)
		}
	}
}

/*
impl Protoable<U256Proto, U256Error> for U256 {
	pub fn as_proto(&self) -> U256Proto {
		U256Proto {
			u0: self.u0,
			u1: self.u1,
			u2: self.u2,
			u3: self.u3,
		}
	}

	pub fn from_proto(proto: &U256Proto) -> Self {
		U256::new(proto.u0, proto.u1, proto.u2, proto.u3)
	}
}
*/

/*
use timeit::*;
use std::sync::atomic::AtomicU64;

#[test]
#[no_mangle]
fn test_u256() {
	let a = U256::new(1, 2, 3, 4);
	let b = U256::new(1, 2, 3, 4);

	let c: [u64; 4] = [1, 2, 3, 4];
	let d: [u64; 4] = [1, 2, 3, 4];

	let s = timeit_loops!(1000, {
		for _ in 0..1000 {
			let i = hamming256(&a, &b);
			core::hint::black_box(&i);
		}
	});

	println!("U256: {:?}ns/loop", s * NS as f64);

	let s = timeit_loops!(1000, {
		for _ in 0..1000 {
			let i = hamming(&c, &d);
			core::hint::black_box(&i);
		}
	});

	println!("Arr256: {:?}ns/loop", s * NS as f64);
}
*/
