//use bytes::Bytes;
//use crypto_bigint::{Encoding, Random, Uint8Array};
//use derive_more::{Display, Error};

use serde::{Deserialize, Serialize};

use crate::*;
pub use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};

/*
   Types
*/
pub type SecretKey = [u8; SECRET_KEY_LENGTH];

pub type PrivateKeyArr = [u8; SECRET_KEY_LENGTH];
pub type DalekXPrivateKeyArr = [u8; SECRET_KEY_LENGTH];
pub type DalekEdPrivateKeyArr = [u8; SECRET_KEY_LENGTH];

pub type PublicKeyArr = [u8; PUBLIC_KEY_LENGTH];
pub type DalekXPublicKeyArr = [u8; PUBLIC_KEY_LENGTH];
pub type DalekEdPublicKeyArr = [u8; PUBLIC_KEY_LENGTH];

pub type SharedSecret = V256;
pub type Signature = V512;

/*
   Errors
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CryptoKeyError {
	EncodingError,
	InvalidYCoordinate,
	InvalidSignature,
	InvalidPublicKey,
	InvalidPrivateKey,
	InvalidKeypair,
	InvalidVanityPrefix,
	FailedToDecompress,
	FailedToCreateDalekEdPublicKey,
	FailedToCreateDalekEdPrivateKey,
}
