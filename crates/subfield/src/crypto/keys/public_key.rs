use crate::*;
use std::convert::From;
use std::iter::Once;
pub use super::common::*;
use std::fmt;

use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

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



#[derive(Clone, Getters, Serialize, Deserialize)]
pub struct PublicKey {
	v256: V256, // edwards public key
	#[serde(skip)]
	ed: OnceCell<DalekEdPublicKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPublicKey>,
}

/**
 * PublicKey
*/
impl PublicKey {
	pub fn new(v256: V256) -> PublicKey {
		PublicKey {
			v256,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	/**
	 * Getters
		*/
	pub fn ed(&self) -> &DalekEdPublicKey {
		self.ed.get_or_init(|| {
			DalekEdPublicKey::from_bytes(&self.v256().bytes().as_slice().try_into().unwrap()).unwrap()
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
			.verify(message, &DalekSignature::from(
				&<[_; SIGNATURE_LENGTH]>::try_from(signature.bytes().as_slice())
					.map_err(|_| KeyError::InvalidSignature)?
			))
		{
			Ok(_) => Ok(true),
			Err(_) => Ok(false),
		}
	}
}

impl HasV256 for PublicKey {
	fn v256(&self) -> &V256 {
		&self.v256
	}
}

/**
 * Hash
*/
impl std::hash::Hash for PublicKey {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.v256.hash(state);
	}
}

/**
 * Stringable
*/
impl Stringable<KeyError> for PublicKey {
	fn to_string(&self) -> String {
		self.v256.to_string()
	}

	fn from_string(string: &str) -> Result<Self, KeyError> {
		Ok(PublicKey::new(
			V256::from_string(string)
				.map_err(|_| KeyError::InvalidPublicKey)?,
		))
	}
}

/**
 * Randomable (nonsense, only used for testing)
*/
impl Randomable for PublicKey {
	fn random() -> Self {
		PublicKey::new(V256::random256())
	}
}

/**
 * Equality
*/
impl PartialEq for PublicKey {
	fn eq(&self, other: &Self) -> bool {
		self.v256.version() == other.v256.version()
			&& self.v256.bytes() == other.v256.bytes()
	}
}

impl Eq for PublicKey {}