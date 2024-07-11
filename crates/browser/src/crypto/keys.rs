use std::convert::From;
use std::iter::Once;

//use bytes::Bytes;
//use crypto_bigint::{Encoding, Random, Uint8Array};
//use derive_more::{Display, Error};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use js_sys::Uint8Array;
use rand::rngs::OsRng;
use rand::RngCore;

use crate::arr::{hamming, xor};
use crate::traits::*;
use ed25519_dalek::{
	Signature as DalekSignature, Signer, SigningKey as DalekEdPrivateKey,
	Verifier, VerifyingKey as DalekEdPublicKey,
};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};
use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;
use x25519_dalek::{
	PublicKey as DalekXPublicKey, SharedSecret as DalekXSharedSecret,
	StaticSecret as DalekXPrivateKey,
};

use super::u256::{self, *};
use crate::arr;

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

pub type SharedSecret = Uint8Array;

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
#[derive(Clone, Getters)]
pub struct PrivateKey {
	#[getset(get = "pub")]
	u256: U256, // edwards25519 private key
	ed: OnceCell<DalekEdPrivateKey>,
	x: OnceCell<DalekXPrivateKey>,
}

#[wasm_bindgen]
#[derive(Clone, Getters)]
pub struct PublicKey {
	#[getset(get = "pub")]
	u256: U256, // edwards25519 public key
	ed: OnceCell<DalekEdPublicKey>,
	x: OnceCell<DalekXPublicKey>,
}

#[wasm_bindgen]
#[derive(Clone, Getters)]
pub struct Keypair {
	public_key: PublicKey,
	private_key: PrivateKey,
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
	pub fn edwards(&self) -> Uint8Array {
		Uint8Array::from(self.u256.unpacked().as_ref())
	}

	#[wasm_bindgen]
	pub fn montgomery(&self) -> Uint8Array {
		Uint8Array::from(self.x().as_bytes().as_ref())
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
	#[wasm_bindgen]
	pub fn toString(&self) -> String {
		self.u256.toString()
	}

	#[wasm_bindgen]
	pub fn fromString(string: &str) -> Result<PublicKey, KeyError> {
		let u256 =
			U256::fromString(string).map_err(|_| KeyError::InvalidPublicKey)?;
		Ok(PublicKey::from_u256(u256))
	}

	#[wasm_bindgen]
	pub fn toBytes(&self) -> Uint8Array {
		self.u256.toBytes()
	}

	#[wasm_bindgen]
	pub fn fromBytes(bytes: &Uint8Array) -> Result<PublicKey, KeyError> {
		let u256 =
			U256::fromBytes(bytes).map_err(|_| KeyError::InvalidPublicKey)?;
		Ok(PublicKey::from_u256(u256))
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

	fn random() -> Self {
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
	pub fn edwards(&self) -> Uint8Array {
		Uint8Array::from(self.u256.unpacked().as_ref())
	}

	#[wasm_bindgen]
	pub fn montgomery(&self) -> Uint8Array {
		Uint8Array::from(self.x().to_bytes().as_ref())
	}

	#[wasm_bindgen]
	pub fn getPublicKey(&self) -> PublicKey {
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
	pub fn sharedSecret(&self, public_key: &PublicKey) -> SharedSecret {
		Uint8Array::from(
			self.x().diffie_hellman(&public_key.x()).as_bytes().as_ref(),
		)
	}

	/**
	 * Conversions
		*/
	#[wasm_bindgen]
	pub fn toString(&self) -> String {
		self.u256.toString()
	}

	#[wasm_bindgen]
	pub fn fromString(string: &str) -> Result<PrivateKey, KeyError> {
		let u256 = U256::fromString(string)
			.map_err(|_| KeyError::InvalidPrivateKey)?;
		Ok(PrivateKey::from_u256(u256))
	}

	#[wasm_bindgen]
	pub fn toBytes(&self) -> Uint8Array {
		self.u256.toBytes()
	}

	#[wasm_bindgen]
	pub fn fromBytes(bytes: &Uint8Array) -> Result<PrivateKey, KeyError> {
		let u256 =
			U256::fromBytes(bytes).map_err(|_| KeyError::InvalidPrivateKey)?;
		Ok(PrivateKey::from_u256(u256))
	}
}

#[wasm_bindgen]
impl Keypair {
	#[wasm_bindgen(constructor)]
	pub fn new(private_key: PrivateKey) -> Result<Keypair, KeyError> {
		let public_key = private_key.getPublicKey();

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
	#[wasm_bindgen]
	pub fn publicKey(&self) -> PublicKey {
		self.public_key.clone()
	}

	#[wasm_bindgen]
	pub fn privateKey(&self) -> PrivateKey {
		self.private_key.clone()
	}

	/**
	 * Operations
		*/
	#[wasm_bindgen]
	pub fn sign(&self, message: &Uint8Array) -> Signature {
		self.privateKey().sign(message)
	}

	#[wasm_bindgen]
	pub fn verify(
		&self,
		message: &Uint8Array,
		signature: &Signature,
	) -> Result<bool, KeyError> {
		self.publicKey().verify(message, signature)
	}

	#[wasm_bindgen]
	pub fn sharedSecret(&self, public_key: &PublicKey) -> SharedSecret {
		self.privateKey().sharedSecret(public_key)
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
			
			let public_key_string = arr::to_base32(&private_key.verifying_key().to_bytes());
			
			if public_key_string.starts_with(prefix) {
				tracing::info!("Found vanity keypair after {} iterations", counter);
				return Ok(Keypair::new(PrivateKey::new(Uint8Array::from(private_key.to_bytes().as_ref())).unwrap()).unwrap());
			}
		}
	}

	/**
	 * Conversions
		*/
	#[wasm_bindgen]
	pub fn toString(&self) -> String {
		self.privateKey().toString() + &self.publicKey().toString()
	}

	#[wasm_bindgen]
	pub fn fromString(string: &str) -> Result<Keypair, KeyError> {
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

	#[wasm_bindgen]
	pub fn toBytes(&self) -> Uint8Array {
		let mut bytes = self.privateKey().toBytes().to_vec();
		bytes.extend(self.publicKey().toBytes().to_vec());
		Uint8Array::from(bytes.as_slice())
	}

	#[wasm_bindgen]
	pub fn fromBytes(bytes: &Uint8Array) -> Result<Keypair, KeyError> {
		let bytes = bytes.to_vec();
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
}
