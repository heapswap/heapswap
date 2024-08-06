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

pub use super::common::*;
use crate::arr;
use crate::vector::*;
use std::fmt;

#[derive(Clone, Getters, Serialize, Deserialize)]
pub struct PublicKey {
	#[getset(get = "pub")]
	u256: U256, // edwards25519 public key
	#[serde(skip)]
	ed: OnceCell<DalekEdPublicKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPublicKey>,
}

impl fmt::Debug for PublicKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("PublicKey")
			.field("u256", &self.u256)
			.finish()
	}
}

/**
 * PublicKey
*/
impl PublicKey {
	pub fn new(ed: PublicKeyArr) -> PublicKey {
		Self::from_u256(U256::new(ed))
	}

	pub fn from_u256(u256: U256) -> Self {
		PublicKey {
			u256,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	/**
	 * Getters
		*/
	pub fn ed(&self) -> &DalekEdPublicKey {
		self.ed.get_or_init(|| {
			DalekEdPublicKey::from_bytes(&self.u256.data_u8()).unwrap()
		})
	}

	pub fn x(&self) -> &DalekXPublicKey {
		self.x.get_or_init(|| {
			DalekXPublicKey::from(self.ed().to_montgomery().to_bytes())
		})
	}

	/**
	 * Operations
		*/

	pub fn verify(
		&self,
		message: &[u8],
		signature: &Signature,
	) -> Result<bool, KeyError> {
		match self.ed().verify(message, &DalekSignature::from(signature)) {
			Ok(_) => Ok(true),
			Err(_) => Ok(false),
		}
	}

}

/**
 * Vecable
*/
impl Vecable<KeyError> for PublicKey {
	fn to_vec(&self) -> Vec<u8> {
		self.u256.to_vec()
	}
	
	fn from_vec(vec: Vec<u8>) -> Result<PublicKey, KeyError> {
		Self::from_arr(&vec)
	}
	
	fn from_arr(arr: &[u8]) -> Result<Self, KeyError> {
		let u256 = U256::from_arr(arr).map_err(|_| KeyError::InvalidPublicKey)?;
		Ok(PublicKey::from_u256(u256))
	}
}

/**
 * Stringable
*/
impl Stringable<KeyError> for PublicKey {
	fn to_string(&self) -> String {
		self.u256.to_string()
	}
	
	fn from_string(string: &str) -> Result<Self, KeyError> {
		let u256 = U256::from_string(string).map_err(|_| KeyError::InvalidPublicKey)?;
		Ok(PublicKey::from_u256(u256))
	}
}