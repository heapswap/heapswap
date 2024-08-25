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

use super::private_key::*;
use super::public_key::*;
use super::{common::*, private_key};
use crate::arr;
use crate::*;

#[derive(Clone, Getters, Serialize, Deserialize)]
#[wasm_bindgen]
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
				return Ok(Keypair::new(PrivateKey::new(V256::new(
					0,
					private_key.to_bytes().as_slice().try_into().unwrap(),
				))));
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

impl Stringable<KeyError> for Keypair {
	fn to_string(&self) -> String {
		self.private_key().to_string()
	}
	
	fn from_string(string: &str) -> Result<Keypair, KeyError> {
		Ok(Keypair::new(PrivateKey::from_string(string)?))
	}
}



#[cfg(feature = "libp2p")]
impl Libp2pKeypairable<KeyError> for Keypair {
	fn to_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair, KeyError> {
		self.private_key().to_libp2p_keypair()
	}

	fn from_libp2p_keypair(
		keypair: libp2p::identity::Keypair,
	) -> Result<Self, KeyError> {
		let private_key = PrivateKey::from_libp2p_keypair(keypair)?;
		let public_key = private_key.public_key();
		Ok(Keypair {
			private_key,
			public_key,
		})
	}
}

impl PartialEq for Keypair {
	fn eq(&self, other: &Self) -> bool {
		self.private_key == other.private_key
			&& self.public_key == other.public_key
	}
}
impl Eq for Keypair {}

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

#[test]
fn test_vanity() {
	let prefix = "aa";
	let keypair = Keypair::vanity(prefix).unwrap();
	assert!(keypair.public_key().to_string().starts_with(prefix));
}

#[cfg(feature = "libp2p")]
#[test]
fn test_libp2p_keypair() {
	let keypair: Keypair = Keypair::random();
	let libp2p_keypair = keypair.to_libp2p_keypair().unwrap();
	let keypair_from_libp2p: Keypair =
		Keypair::from_libp2p_keypair(libp2p_keypair).unwrap();
	assert!(keypair == keypair_from_libp2p);
}


#[wasm_bindgen]
impl Keypair {
	
	/**
	 * Constructors
	*/
	#[wasm_bindgen(constructor)]
	pub fn _js_new(private_key: PrivateKey) -> Keypair {
		Keypair::new(private_key)
	}
	
	#[wasm_bindgen(js_name = "random")]
	pub fn _js_random() -> Keypair {
		Keypair::random()
	}
	
	#[wasm_bindgen(js_name = "vanity")]
	pub async fn _js_vanity(prefix: String) -> Keypair {
		Keypair::vanity(&prefix).unwrap()
	}
	
	/**
	 * Getters
	*/
	#[wasm_bindgen(getter, js_name = "privateKey")]
	pub fn _js_private_key(&self) -> PrivateKey {
		self.private_key().clone()
	}
	
	#[wasm_bindgen(getter, js_name = "publicKey")]
	pub fn _js_public_key(&self) -> PublicKey {
		self.public_key().clone()
	}
	
	/**
	 * Operations
	*/
	#[wasm_bindgen(js_name = "sign")]
	pub fn _js_sign(&self, message: Vec<u8>) -> Signature {
		self.sign(&message)
	}
	
	#[wasm_bindgen(js_name = "verify")]
	pub fn _js_verify(&self, message: Vec<u8>, signature: Signature) -> bool {
		self.verify(&message, &signature).unwrap()
	}
	
	#[wasm_bindgen(js_name = "sharedSecret")]
	pub fn _js_shared_secret(&self, public_key: PublicKey) -> SharedSecret {
		self.shared_secret(&public_key)
	}
	
	/**
	 * Stringable
	*/
	#[wasm_bindgen(js_name = "toString")]
	pub fn _js_to_string(&self) -> String {
		self.to_string()
	}
	#[wasm_bindgen(js_name = "fromString")]
	pub fn _js_from_string(string: String) -> Keypair {
		Keypair::from_string(&string).unwrap()
	}
	
	/**
	 * Byteable
	*/
	#[wasm_bindgen(js_name = "toBytes")]
	pub fn _js_to_bytes(&self) -> Uint8Array {
		serialize(self).unwrap().as_slice().into()
	}
	#[wasm_bindgen(js_name = "fromBytes")]
	pub fn _js_from_bytes(bytes: Uint8Array) -> Keypair {
		deserialize(&bytes.to_vec()).unwrap()
	}
}