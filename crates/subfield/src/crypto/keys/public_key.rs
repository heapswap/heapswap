use crate::*;
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
use crate::versioned_bytes::*;
use std::fmt;

#[derive(Clone, Getters, Serialize, Deserialize)]
pub struct PublicKey {
	data: V256, // edwards public key
	#[serde(skip)]
	ed: OnceCell<DalekEdPublicKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPublicKey>,
}

/**
 * PublicKey
*/
impl PublicKey {
	pub fn new(data: V256) -> PublicKey {
		PublicKey {
			data,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	/**
	 * Getters
		*/

	pub fn data(&self) -> &V256 {
		&self.data
	}

	pub fn ed(&self) -> &DalekEdPublicKey {
		self.ed.get_or_init(|| {
			DalekEdPublicKey::from_bytes(&self.data().data()).unwrap()
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
		match self
			.ed()
			.verify(message, &DalekSignature::from(signature.data()))
		{
			Ok(_) => Ok(true),
			Err(_) => Ok(false),
		}
	}
}

/**
 * Stringable
*/
impl Stringable<KeyError> for PublicKey {
	fn to_string(&self) -> String {
		self.data.to_string()
	}

	fn from_string(string: &str) -> Result<Self, KeyError> {
		Ok(PublicKey::new(
			V256::from_string(string)
				.map_err(|_| KeyError::InvalidPublicKey)?,
		))
	}
}
