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

use crate::arr;
use crate::u256::*;

use super::common::*;
use super::public_key::*;

#[wasm_bindgen]
#[derive(Clone, Getters, Serialize, Deserialize)]
pub struct PrivateKey {
	#[getset(get = "pub")]
	u256: U256, // edwards25519 private key
	#[serde(skip)]
	ed: OnceCell<DalekEdPrivateKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPrivateKey>,
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

	/**
	 * Getters
		*/
	pub fn ed(&self) -> &DalekEdPrivateKey {
		self.ed.get_or_init(|| {
			DalekEdPrivateKey::from_bytes(&self.u256.data_u8())
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

	pub fn sign(&self, message: &[u8]) -> SignatureArr {
		self.ed().sign(message.to_vec().as_slice()).to_bytes()
	}
}

#[wasm_bindgen]
impl PrivateKey {
	#[wasm_bindgen(constructor)]
	pub fn _js_new(ed: EdPrivateKeyJS) -> Result<PrivateKey, KeyError> {
		let ed: PrivateKeyArr = ed
			.to_vec()
			.try_into()
			.map_err(|_| KeyError::InvalidPrivateKey)?;

		Ok(Self::from_u256(U256::new(ed)))
	}

	#[wasm_bindgen]
	pub fn random() -> Self {
		Self::from_u256(U256::random())
	}

	#[wasm_bindgen]
	pub fn edwards(&self) -> U256 {
		U256::new(self.u256.data_u8().clone())
	}

	#[wasm_bindgen]
	pub fn montgomery(&self) -> U256 {
		U256::new(self.x().to_bytes().clone())
	}

	#[wasm_bindgen(js_name = publicKey, getter)]
	pub fn _js_public_key(&self) -> EdPublicKeyJS {
		Uint8Array::from(self.public_key().to_bytes().as_ref())
	}

	/**
	 * Operations
		*/
	#[wasm_bindgen(js_name = sign)]
	pub fn _js_sign(&self, message: &Uint8Array) -> SignatureJS {
		Uint8Array::from(self.sign(message.to_vec().as_slice()).as_ref())
	}

	#[wasm_bindgen(js_name = sharedSecret)]
	pub fn _js_shared_secret(&self, public_key: &PublicKey) -> U256 {
		self.shared_secret(public_key)
	}

	/**
	 * Conversions
		*/
	#[wasm_bindgen]
	pub fn to_string(&self) -> String {
		self.u256.to_string()
	}

	#[wasm_bindgen]
	pub fn from_string(string: &str) -> Result<PrivateKey, KeyError> {
		let u256 = U256::from_string(string)
			.map_err(|_| KeyError::InvalidPrivateKey)?;
		Ok(PrivateKey::from_u256(u256))
	}

	#[wasm_bindgen]
	pub fn to_bytes(&self) -> Vec<u8> {
		self.u256.to_bytes()
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<PrivateKey, KeyError> {
		let u256 =
			U256::from_bytes(bytes).map_err(|_| KeyError::InvalidPrivateKey)?;
		Ok(PrivateKey::from_u256(u256))
	}

	#[wasm_bindgen(js_name = fromBytes)]
	pub fn _js_from_bytes(bytes: &Uint8Array) -> Result<PrivateKey, KeyError> {
		PrivateKey::from_bytes(&bytes.to_vec().as_slice())
	}
}
