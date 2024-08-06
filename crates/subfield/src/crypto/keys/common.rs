use std::convert::From;
use std::iter::Once;

//use bytes::Bytes;
//use crypto_bigint::{Encoding, Random, Uint8Array};
//use derive_more::{Display, Error};
use getset::{CopyGetters, Getters, MutGetters, Setters};

use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::arr::{hamming, xor};
use crate::traits::*;
use ed25519_dalek::{
	Signature as DalekSignature, Signer, SigningKey as DalekEdPrivateKey,
	Verifier, VerifyingKey as DalekEdPublicKey,
};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};
use once_cell::sync::OnceCell;
use x25519_dalek::{
	PublicKey as DalekXPublicKey, SharedSecret as DalekXSharedSecret,
	StaticSecret as DalekXPrivateKey,
};

use crate::arr;
use crate::vector::*;

/**
 * Types
*/
pub type Key = [u8; SECRET_KEY_LENGTH];

pub type PrivateKeyArr = [u8; SECRET_KEY_LENGTH];
pub type DalekXPrivateKeyArr = [u8; SECRET_KEY_LENGTH];
pub type DalekEdPrivateKeyArr = [u8; SECRET_KEY_LENGTH];

pub type PublicKeyArr = [u8; PUBLIC_KEY_LENGTH];
pub type DalekXPublicKeyArr = [u8; PUBLIC_KEY_LENGTH];
pub type DalekEdPublicKeyArr = [u8; PUBLIC_KEY_LENGTH];

pub type Signature = [u8; SIGNATURE_LENGTH];

pub type SharedSecret = U256;

/**
 * Errors
*/
#[derive(Debug)]
pub enum KeyError {
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
