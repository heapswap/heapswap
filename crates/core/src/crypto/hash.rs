use crate::{
    bys,
    traits::{Byteable, Stringable},
};
use bytes::Bytes;

use super::utils::stack_256;

pub type HashArr = [u8; 32];
pub type HashStackedArr = [u64; 4];

pub struct Hash {
    pub bytes: HashArr,
    pub stacked: HashStackedArr,
}

#[derive(Debug)]
pub enum HashError {
    InvalidHash,
}

pub trait Hashing {
    fn hash(data: Bytes) -> Hash;
    fn verify(&self, data: Bytes) -> bool;

    fn to_arr(&self) -> HashArr;
    fn from_arr(bytes: HashArr) -> Hash;

    fn stacked(&self) -> HashStackedArr;
}

impl Hashing for Hash {
    fn hash(data: Bytes) -> Hash {
        let hashed = blake3::hash(&data).into();
        Hash {
            bytes: hashed,
            stacked: stack_256(&hashed),
        }
    }

    fn verify(&self, data: Bytes) -> bool {
        self.stacked() == Hash::hash(data).stacked()
    }

    fn to_arr(&self) -> HashArr {
        self.bytes
    }

    fn from_arr(bytes: HashArr) -> Hash {
        Hash {
            bytes,
            stacked: stack_256(&bytes),
        }
    }

    fn stacked(&self) -> HashStackedArr {
        self.stacked
    }
}

impl Byteable<HashError> for Hash {
    fn to_bytes(&self) -> Bytes {
        Bytes::copy_from_slice(&self.bytes)
    }

    fn from_bytes(bytes: &Bytes) -> Result<Hash, HashError> {
        Ok(Hash::from_arr(bytes.as_ref().try_into().unwrap()))
    }
}

impl Stringable<HashError> for Hash {
    fn to_string(&self) -> String {
        bys::to_base32(&self.to_bytes())
    }

    fn from_string(string: &str) -> Result<Hash, HashError> {
        let bytes = bys::from_base32(&string).map_err(|_| HashError::InvalidHash)?;
        Hash::from_bytes(&bytes)
    }
}
