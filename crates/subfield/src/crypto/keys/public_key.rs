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
use subfield_proto::versioned_bytes::VersionedBytes;

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

#[derive(Clone, Getters)]
pub struct PublicKey {
	versioned_bytes: VersionedBytes, // edwards25519 public key
	ed: OnceCell<DalekEdPublicKey>,
	x: OnceCell<DalekXPublicKey>,
}

/**
 * PublicKey
*/
impl PublicKey {
	pub fn new(versioned_bytes: VersionedBytes) -> PublicKey {
		PublicKey {
			versioned_bytes,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	pub fn from_u256(u256: U256) -> Self {
		Self::new(VersionedBytes {
			version: 0,
			data: u256.to_vec(),
		})
	}

	/**
	 * Getters
		*/
	pub fn version(&self) -> u32 {
		self.versioned_bytes.version
	}

	pub fn versioned_bytes(&self) -> &VersionedBytes {
		&self.versioned_bytes
	}

	pub fn u256(&self) -> &U256 {
		self.versioned_bytes.u256()
	}

	pub fn ed(&self) -> &DalekEdPublicKey {
		self.ed
			.get_or_init(|| DalekEdPublicKey::from_bytes(&self.u256()).unwrap())
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
 * Byteable
*/
impl Byteable<KeyError> for PublicKey {
	fn to_bytes(&self) -> Bytes {
		self.versioned_bytes.to_bytes()
	}

	fn from_bytes(bytes: Bytes) -> Result<Self, KeyError> {
		Ok(PublicKey::new(
			VersionedBytes::from_bytes(bytes)
				.map_err(|_| KeyError::InvalidPublicKey)?,
		))
	}
}

/**
 * Stringable
*/
impl Stringable<KeyError> for PublicKey {
	fn to_string(&self) -> String {
		self.versioned_bytes.to_string()
	}

	fn from_string(string: &str) -> Result<Self, KeyError> {
		Ok(PublicKey::new(
			VersionedBytes::from_string(string)
				.map_err(|_| KeyError::InvalidPublicKey)?,
		))
	}
}
