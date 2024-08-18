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
use std::fmt;
use x25519_dalek::{
	PublicKey as DalekXPublicKey, SharedSecret as DalekXSharedSecret,
	StaticSecret as DalekXPrivateKey,
};

use crate::arr;
use crate::versioned_bytes::*;

use super::common::*;
use super::public_key::*;

#[derive(Clone, Getters, Serialize, Deserialize)]
pub struct PrivateKey {
	data: V256, // edwards private key
	#[serde(skip)]
	ed: OnceCell<DalekEdPrivateKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPrivateKey>,
}

impl PrivateKey {
	pub fn new(data: V256) -> PrivateKey {
		PrivateKey {
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

	pub fn ed(&self) -> &DalekEdPrivateKey {
		self.ed
			.get_or_init(|| DalekEdPrivateKey::from_bytes(&self.data().data()))
	}

	pub fn x(&self) -> &DalekXPrivateKey {
		self.x
			.get_or_init(|| DalekXPrivateKey::from(self.ed().to_scalar_bytes()))
	}

	/**
	 * Operations
		*/

	pub fn public_key(&self) -> PublicKey {
		let public_key = self.ed().verifying_key().to_bytes();
		PublicKey::new(V256::new(*self.data.version(), public_key))
	}

	pub fn shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
		SharedSecret::new(
			*self.data.version(),
			self.x()
				.diffie_hellman(public_key.x())
				.as_bytes()
				.to_vec()
				.try_into()
				.unwrap(),
		)
	}

	pub fn sign(&self, message: &[u8]) -> Signature {
		Signature::new(
			*self.data.version(),
			self.ed()
				.sign(message.to_vec().as_slice())
				.to_bytes()
				.try_into()
				.unwrap(),
		)
	}
}

/**
 * Randomable
*/
impl Randomable for PrivateKey {
	fn random() -> Self {
		PrivateKey::new(V256::random())
	}
}

/**
 * Stringable
*/
impl Stringable<KeyError> for PrivateKey {
	fn to_string(&self) -> String {
		self.data.to_string()
	}

	fn from_string(string: &str) -> Result<Self, KeyError> {
		Ok(PrivateKey::new(
			V256::from_string(string)
				.map_err(|_| KeyError::InvalidPrivateKey)?,
		))
	}
}
