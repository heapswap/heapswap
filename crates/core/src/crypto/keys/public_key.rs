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

pub use super::common::*;
use crate::arr;
use crate::u256::*;

#[wasm_bindgen]
#[derive(Clone, Getters, Serialize, Deserialize)]
pub struct PublicKey {
	#[getset(get = "pub")]
	u256: U256, // edwards25519 public key
	#[serde(skip)]
	ed: OnceCell<DalekEdPublicKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPublicKey>,
}

/**
 * PublicKey
*/
impl PublicKey {
	pub fn new(ed: PublicKeyArr) -> PublicKey {
		Self::from_u256(U256::new(ed))
	}

	pub fn from_u256(u256: U256) -> Self {
		PublicKey {
			u256,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	/**
	 * Getters
		*/
	pub fn ed(&self) -> &DalekEdPublicKey {
		self.ed.get_or_init(|| {
			DalekEdPublicKey::from_bytes(&self.u256.unpacked()).unwrap()
		})
	}

	pub fn x(&self) -> &DalekXPublicKey {
		self.x.get_or_init(|| {
			DalekXPublicKey::from(self.ed().to_montgomery().to_bytes())
		})
	}

	pub fn verify(
		&self,
		message: &[u8],
		signature: &SignatureArr,
	) -> Result<bool, KeyError> {
		match self.ed().verify(message, &DalekSignature::from(signature)) {
			Ok(_) => Ok(true),
			Err(_) => Ok(false),
		}
	}
}

#[wasm_bindgen]
impl PublicKey {
	#[wasm_bindgen(constructor)]
	pub fn _js_new(ed: EdPublicKeyJS) -> Result<PublicKey, KeyError> {
		let ed: PublicKeyArr = ed
			.to_vec()
			.try_into()
			.map_err(|_| KeyError::InvalidPublicKey)?;

		Ok(Self::new(ed))
	}

	#[wasm_bindgen]
	pub fn edwards(&self) -> U256 {
		U256::new(self.u256.unpacked().clone())
	}

	#[wasm_bindgen]
	pub fn montgomery(&self) -> U256 {
		U256::new(self.x().as_bytes().clone())
	}

	/**
	 * Operations
		*/
	#[wasm_bindgen(js_name = verify)]
	pub fn _js_verify(
		&self,
		message: &Uint8Array,
		signature: &Uint8Array,
	) -> Result<bool, KeyError> {
		let message: &[u8] = &message.to_vec();
		let signature: SignatureArr = signature
			.to_vec()
			.as_slice()
			.try_into()
			.map_err(|_| KeyError::InvalidSignature)?;

		self.verify(message, &signature)
	}

	/**
	 * Conversions
		*/
	#[wasm_bindgen(js_name = toString)]
	pub fn to_string(&self) -> String {
		self.u256.to_string()
	}

	#[wasm_bindgen(js_name = fromString)]
	pub fn from_string(string: &str) -> Result<PublicKey, KeyError> {
		let u256 = U256::from_string(string)
			.map_err(|_| KeyError::InvalidPublicKey)?;
		Ok(PublicKey::from_u256(u256))
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		self.u256.to_bytes()
	}

	#[wasm_bindgen(js_name = toBytes)]
	pub fn _js_to_bytes(&self) -> Uint8Array {
		Uint8Array::from(self.to_bytes().as_slice())
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<PublicKey, KeyError> {
		let bytes: PublicKeyArr =
			bytes.try_into().map_err(|_| KeyError::InvalidPublicKey)?;

		let u256 = U256::new(bytes);
		Ok(PublicKey::from_u256(u256))
	}

	#[wasm_bindgen(js_name = fromBytes)]
	pub fn _js_from_bytes(bytes: &Uint8Array) -> Result<PublicKey, KeyError> {
		PublicKey::from_bytes(&bytes.to_vec().as_slice())
	}
}
