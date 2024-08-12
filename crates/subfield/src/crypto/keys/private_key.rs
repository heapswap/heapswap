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
use std::fmt;
use x25519_dalek::{
	PublicKey as DalekXPublicKey, SharedSecret as DalekXSharedSecret,
	StaticSecret as DalekXPrivateKey,
};

use crate::arr;
use crate::versioned_bytes::*;

use super::common::*;
use super::public_key::*;

pub type PrivateKeyBytes = VersionedBytes;

#[derive(Clone, Getters)]
pub struct PrivateKey {
	versioned_bytes: VersionedBytes,
	ed: OnceCell<DalekEdPrivateKey>,
	x: OnceCell<DalekXPrivateKey>,
}

impl PrivateKey {
	pub fn new(versioned_bytes: PrivateKeyBytes) -> PrivateKey {
		PrivateKey {
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

	pub fn random() -> Self {
		Self::new(VersionedBytes::random())
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

	pub fn ed(&self) -> &DalekEdPrivateKey {
		self.ed
			.get_or_init(|| DalekEdPrivateKey::from_bytes(&self.u256()))
	}

	pub fn x(&self) -> &DalekXPrivateKey {
		self.x
			.get_or_init(|| DalekXPrivateKey::from(self.ed().to_scalar_bytes()))
	}

	pub fn shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
		SharedSecret {
			version: self.version(),
			data: self.x().diffie_hellman(public_key.x()).as_bytes().to_vec(),
		}
	}

	pub fn public_key(&self) -> PublicKey {
		let public_key = self.ed().verifying_key().to_bytes();
		PublicKey::from_u256(public_key)
	}

	/**
	 * Operations
		*/

	pub fn sign(&self, message: &[u8]) -> Signature {
		self.ed().sign(message.to_vec().as_slice()).to_bytes()
	}
}

/**
 * Byteable
*/
impl Byteable<KeyError> for PrivateKey {
	fn to_bytes(&self) -> Bytes {
		self.versioned_bytes.to_bytes()
	}

	fn from_bytes(bytes: Bytes) -> Result<Self, KeyError> {
		Ok(PrivateKey::new(
			VersionedBytes::from_bytes(bytes)
				.map_err(|_| KeyError::InvalidPrivateKey)?,
		))
	}
}

/**
 * Stringable
*/
impl Stringable<KeyError> for PrivateKey {
	fn to_string(&self) -> String {
		self.versioned_bytes.to_string()
	}

	fn from_string(string: &str) -> Result<Self, KeyError> {
		Ok(PrivateKey::new(
			VersionedBytes::from_string(string)
				.map_err(|_| KeyError::InvalidPrivateKey)?,
		))
	}
}

/**
 * Libp2pKeypairable
*/
impl Libp2pKeypairable<KeyError> for PrivateKey {
	fn to_libp2p_keypair(&self) -> libp2p::identity::Keypair {
		libp2p::identity::Keypair::ed25519_from_bytes(self.u256().clone())
			.unwrap()
	}

	fn from_libp2p_keypair(
		libp2p_keypair: libp2p::identity::Keypair,
	) -> Result<PrivateKey, KeyError> {
		let ed25519_keypair = libp2p_keypair
			.try_into_ed25519()
			.map_err(|_| KeyError::InvalidPrivateKey)?;
		let private_key = PrivateKey::from_bytes(Bytes::from(
			ed25519_keypair.to_bytes().to_vec(),
		))
		.map_err(|_| KeyError::InvalidPrivateKey)?;
		Ok(private_key)
	}
}
