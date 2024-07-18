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

/**
 * Types
*/
pub type KeyArr = [u8; SECRET_KEY_LENGTH];

pub type PrivateKeyArr = [u8; SECRET_KEY_LENGTH];
pub type DalekXPrivateKeyArr = [u8; SECRET_KEY_LENGTH];
pub type DalekEdPrivateKeyArr = [u8; SECRET_KEY_LENGTH];

pub type PublicKeyArr = [u8; PUBLIC_KEY_LENGTH];
pub type DalekXPublicKeyArr = [u8; PUBLIC_KEY_LENGTH];
pub type DalekEdPublicKeyArr = [u8; PUBLIC_KEY_LENGTH];

pub type SignatureBytes = [u8; SIGNATURE_LENGTH];

pub type Signature = Uint8Array;

pub type EdPublicKey = Uint8Array;
pub type EdPrivateKey = Uint8Array;

pub type XPublicKey = Uint8Array;
pub type XPrivateKey = Uint8Array;

//pub type SharedSecret = Uint8Array;
pub type SharedSecret = U256;

/**
 * Errors
*/
#[wasm_bindgen]
#[derive(Debug)]
pub enum KeyError {
	InvalidYCoordinate,
	InvalidSignature,
	InvalidPublicKey,
	InvalidPrivateKey,
	InvalidKeypair,
	InvalidVanityPrefix,
	FailedToDecompress,
	FailedToCreateDalekEdPublicKey,
	FailedToCreateDalekEdPrivateKey,
}

/**
 * Structs
*/
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

#[wasm_bindgen]
#[derive(Clone, Getters, Serialize, Deserialize)]
pub struct Keypair {
	private_key: PrivateKey,
	public_key: PublicKey,
}

/**
 * PublicKey
*/
#[wasm_bindgen]
impl PublicKey {
	#[wasm_bindgen(constructor)]
	pub fn new(ed: EdPublicKey) -> Result<PublicKey, KeyError> {
		let ed: PublicKeyArr = ed
			.to_vec()
			.try_into()
			.map_err(|_| KeyError::InvalidPublicKey)?;

		let u256 = U256::new(&ed).map_err(|_| KeyError::InvalidPublicKey)?;

		Ok(Self::from_u256(u256))
	}

	fn from_u256(u256: U256) -> Self {
		PublicKey {
			u256,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	/**
	 * Getters
		*/
	fn ed(&self) -> &DalekEdPublicKey {
		self.ed.get_or_init(|| {
			DalekEdPublicKey::from_bytes(&self.u256.unpacked()).unwrap()
		})
	}

	fn x(&self) -> &DalekXPublicKey {
		self.x.get_or_init(|| {
			DalekXPublicKey::from(self.ed().to_montgomery().to_bytes())
		})
	}

	#[wasm_bindgen]
	pub fn edwards(&self) -> U256 {
		U256::new(self.u256.unpacked().as_ref()).unwrap()
	}

	#[wasm_bindgen]
	pub fn montgomery(&self) -> U256 {
		U256::new(self.x().as_bytes().as_ref()).unwrap()
	}

	/**
	 * Operations
		*/
	#[wasm_bindgen]
	pub fn verify(
		&self,
		message: &Uint8Array,
		signature: &Uint8Array,
	) -> Result<bool, KeyError> {
		let message: &[u8] = &message.to_vec();
		let signature: SignatureBytes = signature
			.to_vec()
			.as_slice()
			.try_into()
			.map_err(|_| KeyError::InvalidSignature)?;

		match self.ed().verify(message, &DalekSignature::from(signature)) {
			Ok(_) => Ok(true),
			Err(_) => Ok(false),
		}
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
		let u256 = U256::new(bytes).map_err(|_| KeyError::InvalidPublicKey)?;
		Ok(PublicKey::from_u256(u256))
	}

	#[wasm_bindgen(js_name = fromBytes)]
	pub fn _js_from_bytes(bytes: &Uint8Array) -> Result<PublicKey, KeyError> {
		PublicKey::from_bytes(&bytes.to_vec().as_slice())
	}
}

#[wasm_bindgen]
impl PrivateKey {
	#[wasm_bindgen(constructor)]
	pub fn new(ed: EdPrivateKey) -> Result<PrivateKey, KeyError> {
		let ed: PrivateKeyArr = ed
			.to_vec()
			.try_into()
			.map_err(|_| KeyError::InvalidPrivateKey)?;

		let u256 = U256::new(&ed).map_err(|_| KeyError::InvalidPrivateKey)?;

		Ok(Self::from_u256(u256))
	}

	fn from_u256(u256: U256) -> Self {
		PrivateKey {
			u256,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	#[wasm_bindgen]
	pub fn random() -> Self {
		Self::from_u256(U256::random())
	}

	/**
	 * Getters
		*/
	fn ed(&self) -> &DalekEdPrivateKey {
		self.ed.get_or_init(|| {
			DalekEdPrivateKey::from_bytes(&self.u256.unpacked())
		})
	}

	fn x(&self) -> &DalekXPrivateKey {
		self.x
			.get_or_init(|| DalekXPrivateKey::from(self.ed().to_scalar_bytes()))
	}

	#[wasm_bindgen]
	pub fn edwards(&self) -> U256 {
		U256::new(self.u256.unpacked().as_ref()).unwrap()
	}

	#[wasm_bindgen]
	pub fn montgomery(&self) -> U256 {
		U256::new(self.x().to_bytes().as_ref()).unwrap()
	}

	#[wasm_bindgen(js_name = publicKey, getter)]
	pub fn public_key(&self) -> PublicKey {
		let public_key = self.ed().verifying_key().to_bytes();
		PublicKey::new(Uint8Array::from(public_key.as_ref())).unwrap()
	}

	/**
	 * Operations
		*/
	#[wasm_bindgen]
	pub fn sign(&self, message: &Uint8Array) -> Signature {
		Uint8Array::from(
			self.ed()
				.sign(message.to_vec().as_slice())
				.to_bytes()
				.as_ref(),
		)
	}

	#[wasm_bindgen]
	pub fn shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
		U256::new(self.x().diffie_hellman(&public_key.x()).as_bytes()).unwrap()
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
		let u256 = U256::new(bytes).map_err(|_| KeyError::InvalidPrivateKey)?;
		Ok(PrivateKey::from_u256(u256))
	}

	#[wasm_bindgen(js_name = fromBytes)]
	pub fn _js_from_bytes(bytes: &Uint8Array) -> Result<PrivateKey, KeyError> {
		PrivateKey::from_bytes(&bytes.to_vec().as_slice())
	}
}

#[wasm_bindgen]
impl Keypair {
	#[wasm_bindgen(constructor)]
	pub fn new(private_key: PrivateKey) -> Result<Keypair, KeyError> {
		let public_key = private_key.public_key();

		Ok(Keypair {
			private_key,
			public_key,
		})
	}

	#[wasm_bindgen]
	pub fn random() -> Self {
		let private_key = PrivateKey::random();
		Keypair::new(private_key).unwrap()
	}

	/**
	 * Getters
		*/
	#[wasm_bindgen(js_name = publicKey, getter)]
	pub fn public_key(&self) -> PublicKey {
		self.public_key.clone()
	}

	#[wasm_bindgen(js_name = privateKey, getter)]
	pub fn private_key(&self) -> PrivateKey {
		self.private_key.clone()
	}

	/**
	 * Operations
		*/
	#[wasm_bindgen]
	pub fn sign(&self, message: &Uint8Array) -> Signature {
		self.private_key().sign(message)
	}

	#[wasm_bindgen]
	pub fn verify(
		&self,
		message: &Uint8Array,
		signature: &Signature,
	) -> Result<bool, KeyError> {
		self.public_key().verify(message, signature)
	}

	#[wasm_bindgen(js_name = sharedSecret)]
	pub fn shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
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
				return Ok(Keypair::new(
					PrivateKey::new(Uint8Array::from(
						private_key.to_bytes().as_ref(),
					))
					.unwrap(),
				)
				.unwrap());
			}
		}
	}

	/**
	 * Stringable
		*/
	#[wasm_bindgen]
	pub fn to_string(&self) -> String {
		self.private_key().to_string() + &self.public_key().to_string()
	}

	#[wasm_bindgen]
	pub fn from_string(string: &str) -> Result<Keypair, KeyError> {
		let string_bytes =
			arr::from_base32(string).map_err(|_| KeyError::InvalidKeypair)?;

		let private_key = PrivateKey::from_u256(
			U256::new(&string_bytes[..SECRET_KEY_LENGTH])
				.map_err(|_| KeyError::InvalidPrivateKey)?,
		);

		let public_key = PublicKey::from_u256(
			U256::new(&string_bytes[SECRET_KEY_LENGTH..])
				.map_err(|_| KeyError::InvalidPublicKey)?,
		);

		Keypair::new(private_key).map_err(|_| KeyError::InvalidKeypair)
	}

	/**
	 * Byteable
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
		let private_key = PrivateKey::from_u256(
			U256::new(&bytes[..SECRET_KEY_LENGTH])
				.map_err(|_| KeyError::InvalidPrivateKey)?,
		);
		let public_key = PublicKey::from_u256(
			U256::new(&bytes[SECRET_KEY_LENGTH..])
				.map_err(|_| KeyError::InvalidPublicKey)?,
		);

		Keypair::new(private_key).map_err(|_| KeyError::InvalidKeypair)
	}

	#[wasm_bindgen(js_name = fromBytes)]
	pub fn _js_from_bytes(bytes: &Uint8Array) -> Result<Keypair, KeyError> {
		Keypair::from_bytes(&bytes.to_vec().as_slice())
	}
}
