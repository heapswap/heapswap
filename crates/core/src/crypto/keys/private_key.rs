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
use crate::u256::*;

use super::common::*;
use super::public_key::*;

#[derive(Clone, Getters, Serialize, Deserialize)]
pub struct PrivateKey {
	#[getset(get = "pub")]
	u256: U256, // edwards25519 private key
	#[serde(skip)]
	ed: OnceCell<DalekEdPrivateKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPrivateKey>,
}

impl fmt::Debug for PrivateKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("PrivateKey")
			.field("u256", &self.u256)
			.finish()
	}
}

impl PrivateKey {
	pub fn new(ed: PrivateKeyArr) -> PrivateKey {
		Self::from_u256(U256::new(ed))
	}

	pub fn from_u256(u256: U256) -> Self {
		PrivateKey {
			u256,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	pub fn random() -> Self {
		Self::from_u256(U256::random())
	}

	/**
	 * Getters
		*/
	pub fn ed(&self) -> &DalekEdPrivateKey {
		self.ed.get_or_init(|| {
			DalekEdPrivateKey::from_bytes(&self.u256.unpacked())
		})
	}

	pub fn x(&self) -> &DalekXPrivateKey {
		self.x
			.get_or_init(|| DalekXPrivateKey::from(self.ed().to_scalar_bytes()))
	}

	pub fn shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
		U256::new(self.x().diffie_hellman(public_key.x()).as_bytes().clone())
	}

	pub fn public_key(&self) -> PublicKey {
		let public_key = self.ed().verifying_key().to_bytes();
		PublicKey::new(public_key)
	}

	/**
	 * Operations
		*/

	pub fn sign(&self, message: &[u8]) -> Signature {
		self.ed().sign(message.to_vec().as_slice()).to_bytes()
	}

	/**
	 * Conversions
		*/
	pub fn to_string(&self) -> String {
		self.u256.to_string()
	}

	pub fn from_string(string: &str) -> Result<PrivateKey, KeyError> {
		let u256 = U256::from_string(string)
			.map_err(|_| KeyError::InvalidPrivateKey)?;
		Ok(PrivateKey::from_u256(u256))
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		self.u256.to_bytes()
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<PrivateKey, KeyError> {
		let u256 =
			U256::from_bytes(bytes).map_err(|_| KeyError::InvalidPrivateKey)?;
		Ok(PrivateKey::from_u256(u256))
	}
}

impl Libp2pKeypairable<KeyError> for PrivateKey {
	fn to_libp2p_keypair(&self) -> libp2p::identity::Keypair {
		libp2p::identity::Keypair::ed25519_from_bytes(
			self.u256().unpacked().clone(),
		)
		.unwrap()
	}

	fn from_libp2p_keypair(
		libp2p_keypair: libp2p::identity::Keypair,
	) -> Result<PrivateKey, KeyError> {
		let ed25519_keypair = libp2p_keypair
			.try_into_ed25519()
			.map_err(|_| KeyError::InvalidPrivateKey)?;
		let private_key = PrivateKey::from_bytes(&ed25519_keypair.to_bytes())
			.map_err(|_| KeyError::InvalidPrivateKey)?;
		Ok(private_key)
	}
}
