use crate::{
    bys, comparison,
    traits::{Arrable, Byteable, Randomable, Stringable},
};
use bytes::Bytes;
use rand::RngCore;
use std::cmp::Ordering;

/**
 * Errors
*/
#[derive(Debug, PartialEq)]
pub enum U256Error {
    InvalidLength,
}

pub type Arr256 = [u8; 32];

/**
 * Structs
*/
#[derive(Debug, Clone, Copy)]
pub struct U256 {
    u0: u64,
    u1: u64,
    u2: u64,
    u3: u64,
    pub arr: Arr256,
}

impl U256 {
    pub fn new(u0: u64, u1: u64, u2: u64, u3: u64) -> Self {
        let mut arr = [0u8; 32];
        arr[0..8].copy_from_slice(&u0.to_le_bytes());
        arr[8..16].copy_from_slice(&u1.to_le_bytes());
        arr[16..24].copy_from_slice(&u2.to_le_bytes());
        arr[24..32].copy_from_slice(&u3.to_le_bytes());

        U256 {
            u0,
            u1,
            u2,
            u3,
            arr,
        }
    }
}

/**
 * Conversions
*/

impl Arrable<Arr256, U256Error> for U256 {
    fn to_arr(&self) -> Arr256 {
        self.arr
    }

    fn from_arr(arr: &Arr256) -> Result<Self, U256Error> {
        Ok(U256 {
            u0: u64::from_le_bytes(arr[0..8].try_into().unwrap()),
            u1: u64::from_le_bytes(arr[8..16].try_into().unwrap()),
            u2: u64::from_le_bytes(arr[16..24].try_into().unwrap()),
            u3: u64::from_le_bytes(arr[24..32].try_into().unwrap()),
            arr: arr.clone(),
        })
    }
}

impl Byteable<U256Error> for U256 {
    fn to_bytes(&self) -> Bytes {
        Bytes::copy_from_slice(&self.arr)
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
        bys::to_base32(&self.to_bytes())
    }

    fn from_string(string: &str) -> Result<Self, U256Error> {
        let bytes = bys::from_base32(string).map_err(|_| U256Error::InvalidLength)?;

        U256::from_bytes(&bytes)
    }
}

impl Randomable<U256Error> for U256 {
    fn random() -> Result<Self, U256Error> {
        let mut arr = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut arr);
        U256::from_arr(&arr)
    }
}

/**
 * Ordering
*/
impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.u0 != other.u0 {
            Some(self.u0.cmp(&other.u0))
        } else if self.u1 != other.u1 {
            Some(self.u1.cmp(&other.u1))
        } else if self.u2 != other.u2 {
            Some(self.u2.cmp(&other.u2))
        } else {
            Some(self.u3.cmp(&other.u3))
        }
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for U256 {
    fn eq(&self, other: &Self) -> bool {
        self.u0 == other.u0 && self.u1 == other.u1 && self.u2 == other.u2 && self.u3 == other.u3
    }
}

impl Eq for U256 {}

/**
 * Operations
*/
// Specialized xor for 256-bit integers as U256
pub fn xor256(a: &U256, b: &U256) -> U256 {
    U256 {
        u0: a.u0 ^ b.u0,
        u1: a.u1 ^ b.u1,
        u2: a.u2 ^ b.u2,
        u3: a.u3 ^ b.u3,
        //arr: comparison::xor(&a.to_arr(),&b.to_arr()),
        arr: [0; 32], // don't need to compute the array because computations are not serialized, might need to remove this
    }
}

// Specialized hamming for 256-bit integers as U256
pub fn hamming256(a: &U256, b: &U256) -> u32 {
    (a.u0 ^ b.u0).count_ones()
        + (a.u1 ^ b.u1).count_ones()
        + (a.u2 ^ b.u2).count_ones()
        + (a.u3 ^ b.u3).count_ones()
}
