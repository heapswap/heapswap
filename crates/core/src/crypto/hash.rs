use crate::{
    bys,
    traits::{Arrable, Byteable, Stringable},
    u256::*,
};
use bytes::Bytes;

type Hash = U256;

#[derive(Debug)]
pub enum HashError {
    InvalidHash,
}

pub trait Hashing {
    fn hash(data: Bytes) -> Hash;
    fn verify(&self, data: Bytes) -> bool;
}

impl Hashing for Hash {
    fn hash(data: Bytes) -> Hash {
        Hash::from_arr(&blake3::hash(&data).into()).unwrap()
    }

    fn verify(&self, data: Bytes) -> bool {
        self == &Hash::hash(data)
    }
}
