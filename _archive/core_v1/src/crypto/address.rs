use crate::{arr, arr::*, bys, constants::NS, traits::*};
use bytes::Bytes;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use rand::RngCore;
use std::fmt;
use std::{cmp::Ordering, ops::BitXor};
use std::{collections::HashMap, sync::Mutex};
//use heapswap_protos::uAddress::Address as AddressProto;
use super::hash::*;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;

#[derive(Debug, strum::Display, PartialEq)]
pub enum AddressError {
	InvalidBase32,
	InvalidLength,
}

const SEED_ADDRESS_LENGTH: usize = 32;
const UNPACKED_ADDRESS_LENGTH: usize = 64;
const PACKED_ADDRESS_LENGTH: usize = UNPACKED_ADDRESS_LENGTH / 8;

pub type SeedAddress = [u8; SEED_ADDRESS_LENGTH]; // 256 bits - ed25519 key
pub type UnpackedAddress = [u8; UNPACKED_ADDRESS_LENGTH]; // 384 bits - ed25519 key + blake3 hash of key
pub type PackedAddress = [u64; (PACKED_ADDRESS_LENGTH) as usize]; // 384 bits - packed to u64 for more efficient operations

#[derive(Debug, Clone, Getters)]
pub struct Address {
	seed: SeedAddress,
	popcnt: OnceCell<u32>,
	packed: OnceCell<PackedAddress>,
	unpacked: OnceCell<UnpackedAddress>,
}

/**
 * Serialization - display as string
*/
impl Serialize for Address {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let string_repr = self.to_string();
		serializer.serialize_str(&string_repr)
	}
}

impl<'de> Deserialize<'de> for Address {
	fn deserialize<D>(deserializer: D) -> Result<Address, D::Error>
	where
		D: Deserializer<'de>,
	{
		let string_repr = String::deserialize(deserializer)?;
		Address::from_string(&string_repr).map_err(serde::de::Error::custom)
	}
}

/**
 * Packing
*/
fn unpack(packed: PackedAddress) -> UnpackedAddress {
	let mut unpacked = [0u8; UNPACKED_ADDRESS_LENGTH];
	for i in 0..UNPACKED_ADDRESS_LENGTH {
		unpacked[i * 8..(i + 1) * 8].copy_from_slice(&packed[i].to_le_bytes());
	}
	unpacked
}

fn pack(unpacked: &UnpackedAddress) -> PackedAddress {
	let mut packed = [0u64; PACKED_ADDRESS_LENGTH];
	for i in 0..PACKED_ADDRESS_LENGTH {
		packed[i] = u64::from_le_bytes(
			unpacked[i * 8..(i + 1) * 8].try_into().unwrap(),
		);
	}
	packed
}

impl Address {
	pub fn new(seed: SeedAddress) -> Self {
		Address::from_seed(seed)
	}

	pub fn from_seed(seed: SeedAddress) -> Self {
		Address {
			popcnt: OnceCell::new(),
			seed: seed,
			packed: OnceCell::new(),
			unpacked: OnceCell::new(),
		}
	}

	pub fn popcnt(&self) -> u32 {
		*self.popcnt.get_or_init(|| popcount(self.packed()))
	}

	pub fn to_seed(&self) -> SeedAddress {
		self.seed
	}

	pub fn seed(&self) -> &SeedAddress {
		&self.seed
	}

	pub fn unpacked(&self) -> &UnpackedAddress {
		self.unpacked.get_or_init(|| {
			[self.seed, super::hash::hash(&self.seed).to_arr()]
				.concat()
				.try_into()
				.unwrap()
		})
	}

	pub fn packed(&self) -> &PackedAddress {
		self.packed.get_or_init(|| pack(self.data_u8()))
	}

	pub fn from_unpacked(unpacked: UnpackedAddress) -> Self {
		let seed: [u8; SEED_ADDRESS_LENGTH] =
			unpacked[..SEED_ADDRESS_LENGTH].try_into().unwrap();

		Address {
			popcnt: OnceCell::new(),
			seed: seed,
			packed: OnceCell::new(),
			unpacked: OnceCell::from(unpacked),
		}
	}

	pub fn from_packed(packed: PackedAddress) -> Self {
		let unpacked = unpack(packed);
		let seed = unpacked[..32].try_into().unwrap();

		Address {
			popcnt: OnceCell::new(),
			seed,
			packed: OnceCell::from(packed),
			unpacked: OnceCell::from(unpacked),
		}
	}

	// takes 9ns on release, 64 nodes should take ~.6ms
	pub fn jaccard(&self, other: &Address) -> f64 {
		let intersection = andcount(self.packed(), other.packed()) as f64;
		let union = self.popcnt() as f64 + other.popcnt() as f64 - intersection;

		intersection / union
	}

	pub fn zero() -> Self {
		Address::new([0; SEED_ADDRESS_LENGTH])
	}
}

impl Default for Address {
	fn default() -> Self {
		Address::zero()
	}
}

/**
 * Ordering  
*/
impl PartialOrd for Address {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.packed()
			.iter()
			.zip(other.packed().iter())
			.find_map(|(a, b)| if a != b { Some(a.cmp(b)) } else { None })
			.or(Some(Ordering::Equal))
	}
}

impl Ord for Address {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(other).unwrap()
	}
}

impl PartialEq for Address {
	fn eq(&self, other: &Self) -> bool {
		self.packed()
			.iter()
			.zip(other.packed().iter())
			.all(|(a, b)| a == b)
	}
}

impl Eq for Address {}

/**
 * Xor
*/

impl BitXor for Address {
	type Output = Self;

	fn bitxor(self, rhs: Self) -> Self {
		Address::from_packed(xor(&self.packed(), &rhs.packed()))
	}
}

/**
 * Conversions
*/

impl Randomable for Address {
	fn random() -> Self {
		let mut unpacked = [0u8; UNPACKED_ADDRESS_LENGTH];
		rand::rngs::OsRng.fill_bytes(&mut unpacked);
		Address::from_unpacked(unpacked)
	}
}

impl Arrable<SeedAddress, AddressError> for Address {
	fn to_arr(&self) -> SeedAddress {
		*self.seed()
	}

	fn from_arr(seed: &SeedAddress) -> Result<Self, AddressError> {
		Ok(Address::from_seed(*seed))
	}
}

impl Byteable<AddressError> for Address {
	fn to_bytes(&self) -> Bytes {
		Bytes::copy_from_slice(&self.to_arr())
	}

	fn from_bytes(bytes: &Bytes) -> Result<Self, AddressError> {
		if bytes.len() != UNPACKED_ADDRESS_LENGTH {
			return Err(AddressError::InvalidLength);
		}
		Ok(Address::from_arr(&bytes[..].try_into().unwrap()).unwrap())
	}
}

impl Stringable<AddressError> for Address {
	fn to_string(&self) -> String {
		arr::to_base32(&self.to_arr())
	}

	fn from_string(string: &str) -> Result<Self, AddressError> {
		let vec = arr::from_base32(string)
			.map_err(|_| AddressError::InvalidLength)?;

		if vec.len() == SEED_ADDRESS_LENGTH {
			let mut arr = [0u8; SEED_ADDRESS_LENGTH];
			arr.copy_from_slice(&vec);
			Ok(Address::from_arr(&arr)?)
		} else {
			Err(AddressError::InvalidLength)
		}
	}
}

#[test]
fn test_address() -> Result<(), AddressError> {
	let a = Address::random();
	let b = Address::random();

	// time the jaccard distance calculation
	let s = timeit::timeit_loops!(1000, {
		let i = a.jaccard(&b);
		core::hint::black_box(&i);
	});

	println!("Jaccard: {:?}ns/loop", s * NS as f64);

	//const TEST_ADDRESS_COUNT: usize = 10_000_000;
	const TEST_ADDRESS_COUNT: usize = 10_000;

	// generate random addresses
	let addresses: Vec<Address> =
		(0..TEST_ADDRESS_COUNT).map(|_| Address::random()).collect();

	let query = Address::random();

	// find nearest n jaccard addresses
	const NEAREST_N: usize = 10;
	let mut address_distances = addresses
		.iter()
		.map(|address| (address, address.jaccard(&query)))
		.collect::<Vec<_>>();

	let mut nearest_nodes = Vec::with_capacity(NEAREST_N);

	for _ in 0..NEAREST_N {
		let mut nearest_distance = 0.;
		let mut nearest_distance_index = 0;
		for i in 0..address_distances.len() {
			let (_, distance) = &address_distances[i];
			if distance > &nearest_distance {
				nearest_distance = *distance;
				nearest_distance_index = i;
			}
		}

		nearest_nodes.push(address_distances.remove(nearest_distance_index));
	}

	// print the nearest addresses and their jaccard distances
	for (address, distance) in nearest_nodes {
		println!("{}: {}", address.to_string(), distance);
	}

	Ok(())
}
