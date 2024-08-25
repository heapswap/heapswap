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
use crate::*;

use super::common::*;
use super::public_key::*;

#[derive(Clone, Getters, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct PrivateKey {
	v256: V256, // edwards private key
	#[serde(skip)]
	ed: OnceCell<DalekEdPrivateKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPrivateKey>,
}

impl PrivateKey {
	pub fn new(v256: V256) -> PrivateKey {
		PrivateKey {
			v256,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	/**
	 * Getters
		*/

	pub fn ed(&self) -> &DalekEdPrivateKey {
		self.ed
			.get_or_init(|| DalekEdPrivateKey::from_bytes(
				 &self.v256().bytes().as_slice().try_into().unwrap()
			))
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
		PublicKey::new(V256::new(*self.v256.version(), public_key.as_slice()))
	}

	pub fn shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
		SharedSecret::new(
			*self.v256.version(),
			self.x()
				.diffie_hellman(public_key.x())
				.as_bytes().as_slice().try_into().unwrap()
		)
	}

	pub fn sign(&self, message: &[u8]) -> Signature {
		Signature::new(
			*self.v256.version(),
			self.ed()
				.sign(message.to_vec().as_slice())
				.to_bytes()
				.as_slice().try_into().unwrap(),
		)
	}
}

/**
 * Randomable
*/
impl Randomable for PrivateKey {
	fn random() -> Self {
		PrivateKey::new(V256::random256())
	}
}

/**
 * Hash
*/
impl std::hash::Hash for PrivateKey {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.v256.hash(state);
	}
}

/**
 * Stringable
*/
impl Stringable<KeyError> for PrivateKey {
	fn to_string(&self) -> String {
		self.v256.to_string()
	}

	fn from_string(string: &str) -> Result<Self, KeyError> {
		Ok(PrivateKey::new(
			V256::from_string(string)
				.map_err(|_| KeyError::InvalidPrivateKey)?,
		))
	}
}

/**
 * Libp2pKeypairable
*/
#[cfg(feature = "libp2p")]
impl Libp2pKeypairable<KeyError> for PrivateKey {
	fn to_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair, KeyError> {
		Ok(libp2p::identity::Keypair::ed25519_from_bytes(
			self.v256().bytes().clone(),
		)
		.map_err(|_| KeyError::InvalidPrivateKey)?)
		// .to_protobuf_encoding()
		// .map_err(|_| KeyError::EncodingError)?
		// .to_vec()
		// .as_slice())
	}

	fn from_libp2p_keypair(
		keypair: libp2p::identity::Keypair,
	) -> Result<Self, KeyError> {
		let private_key_bytes = keypair
			.try_into_ed25519()
			.map_err(|_| KeyError::InvalidPrivateKey)?
			.to_bytes();
		let private_key_bytes: [u8; SECRET_KEY_LENGTH] =
			private_key_bytes[..SECRET_KEY_LENGTH].try_into().unwrap();
		let private_key = PrivateKey::new(V256::new(0, private_key_bytes));
		let public_key = private_key.public_key();

		Ok(private_key)
	}
}

/**
 * HasV256
*/

impl HasV256 for PrivateKey {
	fn v256(&self) -> &V256 {
		&self.v256
	}
}

/**
 * Equality
*/
impl PartialEq for PrivateKey {
	fn eq(&self, other: &Self) -> bool {
		self.v256.version() == other.v256.version()
			&& self.v256.bytes() == other.v256.bytes()
	}
}

impl Eq for PrivateKey {}
