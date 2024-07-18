use std::convert::From;
use std::iter::Once;

//use bytes::Bytes;
//use crypto_bigint::{Encoding, Random, Uint8Array};
//use derive_more::{Display, Error};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

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
use crate::u256::*;

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

pub type SignatureArr = [u8; SIGNATURE_LENGTH];

pub type SignatureJS = Uint8Array;

pub type EdPublicKeyJS = Uint8Array;
pub type EdPrivateKeyJS = Uint8Array;

pub type XPublicKeyJS = Uint8Array;
pub type XPrivateKeyJS = Uint8Array;

pub type SharedSecret = U256;

/**
 * Errors
*/
#[wasm_bindgen]
#[derive(Debug)]
pub enum KeyError {
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
