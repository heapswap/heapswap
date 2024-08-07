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

use super::private_key::*;
use super::public_key::*;
use super::{common::*, private_key};
use crate::arr;
use crate::vector::*;

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct Keypair {
	#[getset(get = "pub")]
	private_key: PrivateKey,
	#[getset(get = "pub")]
	public_key: PublicKey,
}

impl Keypair {
	/**
	 * Constructors
		*/
	pub fn new(private_key: PrivateKey) -> Keypair {
		let public_key = private_key.public_key();

		Keypair {
			private_key,
			public_key,
		}
	}

	pub fn random() -> Self {
		let private_key = PrivateKey::random();
		Keypair::new(private_key)
	}

	pub fn vanity(prefix: &str) -> Result<Keypair, KeyError> {
		// test if prefix is valid base32
		let _ = arr::from_base32(prefix)
			.map_err(|_| KeyError::InvalidVanityPrefix)?;

		let mut counter = 0;
		loop {
			counter += 1;

			let private_key = DalekEdPrivateKey::generate(&mut OsRng);

			let public_key_string =
				arr::to_base32(&private_key.verifying_key().to_bytes());

			if public_key_string.starts_with(prefix) {
				return Ok(Keypair::new(PrivateKey::new(
					private_key.to_bytes(),
				)));
			}
		}
	}

	/**
	 * Operations
		*/

	pub fn sign(&self, message: &[u8]) -> Signature {
		self.private_key().sign(message)
	}

	pub fn verify(
		&self,
		message: &[u8],
		signature: &Signature,
	) -> Result<bool, KeyError> {
		self.public_key().verify(message, signature)
	}

	pub fn shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
		self.private_key().shared_secret(public_key)
	}
}

/**
 * Stringable
*/
impl Stringable<KeyError> for Keypair {
	fn to_string(&self) -> String {
		arr::to_base32(&self.to_vec())
	}

	fn from_string(string: &str) -> Result<Keypair, KeyError> {
		let string_bytes =
			arr::from_base32(string).map_err(|_| KeyError::InvalidKeypair)?;

		Self::from_arr(&string_bytes)
	}
}

/**
 * Vecable
*/
impl Vecable<KeyError> for Keypair {
	fn to_vec(&self) -> Vec<u8> {
		let mut bytes = self.private_key().to_vec().to_vec();
		bytes.extend(self.public_key().to_vec().to_vec());
		bytes
	}

	fn from_vec(bytes: Vec<u8>) -> Result<Keypair, KeyError> {
		Self::from_arr(&bytes)
	}
	
	fn from_arr(arr: &[u8]) -> Result<Keypair, KeyError> {
		let private_key = PrivateKey::from_arr(&arr[..SECRET_KEY_LENGTH])
			.map_err(|_| KeyError::InvalidPrivateKey)?;
		let public_key = private_key.public_key();

		Ok(Keypair::new(private_key))
	}
}

/**
 * Libp2pKeypairable
*/
impl Libp2pKeypairable<KeyError> for Keypair {
	fn to_libp2p_keypair(&self) -> libp2p::identity::Keypair {
		self.private_key().to_libp2p_keypair()
	}

	fn from_libp2p_keypair(
		libp2p_keypair: libp2p::identity::Keypair,
	) -> Result<Keypair, KeyError> {
		PrivateKey::from_libp2p_keypair(libp2p_keypair)
			.map(|private_key| Keypair::new(private_key))
	}
}

#[test]
fn test_sign_and_verify() {
	let keypair = Keypair::random();
	let message = b"hello world";
	let signature = keypair.sign(message);
	assert!(keypair.public_key().verify(message, &signature).unwrap());

}

#[test]
fn test_shared_secret() {
	let alice = Keypair::random();
	let bob = Keypair::random();
	let alice_shared_secret = alice.shared_secret(&bob.public_key());
	let bob_shared_secret = bob.shared_secret(&alice.public_key());
	assert_eq!(alice_shared_secret, bob_shared_secret);
}