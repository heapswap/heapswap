use std::convert::From;
use std::iter::Once;

//use bytes::Bytes;
//use crypto_bigint::{Encoding, Random, Uint8Array};
//use derive_more::{Display, Error};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

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
use crate::u256::*;

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct Keypair {
	#[getset(get = "pub")]
	private_key: PrivateKey,
	#[getset(get = "pub")]
	public_key: PublicKey,
}

impl Keypair {
	pub fn new(private_key: PrivateKey) -> Keypair {
		let public_key = private_key.public_key();

		Keypair {
			private_key,
			public_key,
		}
	}

	/**
	 * Operations
		*/

	pub fn sign(&self, message: &[u8]) -> SignatureArr {
		self.private_key().sign(message)
	}

	pub fn verify(
		&self,
		message: &[u8],
		signature: &SignatureArr,
	) -> Result<bool, KeyError> {
		self.public_key().verify(message, signature)
	}

	pub fn shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
		self.private_key().shared_secret(public_key)
	}
}

#[wasm_bindgen]
impl Keypair {
	#[wasm_bindgen(constructor)]
	pub fn _js_new(private_key: PrivateKey) -> Result<Keypair, KeyError> {
		let public_key = private_key.public_key();

		Ok(Keypair {
			private_key,
			public_key,
		})
	}

	#[wasm_bindgen]
	pub fn random() -> Self {
		let private_key = PrivateKey::random();
		Keypair::new(private_key)
	}

	/**
	 * Getters
		*/
	#[wasm_bindgen(js_name = publicKey, getter)]
	pub fn _js_public_key(&self) -> PublicKey {
		self.public_key.clone()
	}

	#[wasm_bindgen(js_name = privateKey, getter)]
	pub fn _js_private_key(&self) -> PrivateKey {
		self.private_key.clone()
	}

	/**
	 * Operations
		*/
	#[wasm_bindgen(js_name = sign)]
	pub fn _js_sign(&self, message: &Uint8Array) -> SignatureJS {
		self.private_key()._js_sign(message)
	}

	#[wasm_bindgen(js_name = verify)]
	pub fn _js_verify(
		&self,
		message: &Uint8Array,
		signature: &SignatureJS,
	) -> Result<bool, KeyError> {
		self.public_key()._js_verify(message, signature)
	}

	#[wasm_bindgen(js_name = sharedSecret)]
	pub fn _js_shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
		self.private_key().shared_secret(public_key)
	}

	#[wasm_bindgen]
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
	 * Stringable
		*/
	#[wasm_bindgen]
	pub fn to_string(&self) -> String {
		arr::to_base32(&self.to_bytes())
	}

	#[wasm_bindgen]
	pub fn from_string(string: &str) -> Result<Keypair, KeyError> {
		let string_bytes =
			arr::from_base32(string).map_err(|_| KeyError::InvalidKeypair)?;

		Self::from_bytes(&string_bytes)
	}

	/**
	 * Conversions
		*/

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut bytes = self.private_key().to_bytes().to_vec();
		bytes.extend(self.public_key().to_bytes().to_vec());
		bytes
	}

	#[wasm_bindgen(js_name = toBytes)]
	pub fn _js_to_bytes(&self) -> Uint8Array {
		Uint8Array::from(self.to_bytes().to_vec().as_slice())
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Keypair, KeyError> {
		let private_key = PrivateKey::from_bytes(&bytes[..SECRET_KEY_LENGTH])
			.map_err(|_| KeyError::InvalidPrivateKey)?;
		let public_key = PublicKey::from_bytes(&bytes[SECRET_KEY_LENGTH..])
			.map_err(|_| KeyError::InvalidPublicKey)?;

		Ok(Keypair::new(private_key))
	}

	#[wasm_bindgen(js_name = fromBytes)]
	pub fn _js_from_bytes(bytes: &Uint8Array) -> Result<Keypair, KeyError> {
		Keypair::from_bytes(&bytes.to_vec().as_slice())
	}
}
