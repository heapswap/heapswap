use std::convert::From;
use std::iter::Once;

use bytes::Bytes;
//use crypto_bigint::{Encoding, Random, Address};
use derive_more::{Display, Error};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use timeit::*;

use super::address::*;
use crate::arr::{hamming, xor};
use crate::bys;
use crate::traits::*;
use ed25519_dalek::{
	Signature, Signer, SigningKey as DalekEdPrivateKey, Verifier,
	VerifyingKey as DalekEdPublicKey,
};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};
use once_cell::sync::OnceCell;
use x25519_dalek::{
	PublicKey as DalekXPublicKey, SharedSecret as DalekXSharedSecret,
	StaticSecret as DalekXPrivateKey,
};

/**
 * Types
*/
pub type KeyArr = [u8; SECRET_KEY_LENGTH];

type PrivateKeyArr = [u8; SECRET_KEY_LENGTH];
type DalekXPrivateKeyArr = [u8; SECRET_KEY_LENGTH];
type DalekEdPrivateKeyArr = [u8; SECRET_KEY_LENGTH];

type PublicKeyArr = [u8; PUBLIC_KEY_LENGTH];
type DalekXPublicKeyArr = [u8; PUBLIC_KEY_LENGTH];
type DalekEdPublicKeyArr = [u8; PUBLIC_KEY_LENGTH];

type SignatureBytes = [u8; SIGNATURE_LENGTH];

type EdPublicKey = Address;
type EdPrivateKey = Address;

type XPublicKey = Address;

/**
 * Errors
*/

#[derive(Debug, Display, Error)]
pub enum KeyError {
	InvalidYCoordinate,
	InvalidSignature,
	InvalidPublicKey,
	InvalidPrivateKey,
	InvalidKeyPair,
	FailedToDecompress,
	FailedToCreateDalekEdPublicKey,
	FailedToCreateDalekEdPrivateKey,
}

/**
 * Structs
*/
#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct PrivateKey {
	#[getset(get = "pub")]
	address: Address, // edwards25519 private key
	#[serde(skip)]
	ed: OnceCell<DalekEdPrivateKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPrivateKey>,
}

#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct PublicKey {
	#[getset(get = "pub")]
	address: Address, // edwards25519 public key
	#[serde(skip)]
	ed: OnceCell<DalekEdPublicKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPublicKey>,
}

#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct KeyPair {
	#[getset(get = "pub")]
	public_key: PublicKey,
	#[getset(get = "pub")]
	private_key: PrivateKey,
}

/**
 * Traits
*/
pub trait Signable {
	fn sign(&self, message: &Bytes) -> Signature;
}

pub trait Verifiable {
	fn verify(
		&self,
		message: &Bytes,
		signature: &Signature,
	) -> Result<(), KeyError>;
}

pub trait SharedSecretable {
	fn shared_secret(&self, public_key: &PublicKey) -> DalekXSharedSecret;
}

/**
 * PublicKey
*/

impl PublicKey {
	pub fn new(ed_arr: DalekEdPublicKeyArr) -> Result<Self, KeyError> {
		let address = Address::from_arr(&ed_arr)
			.map_err(|_| KeyError::InvalidPublicKey)?;

		Self::from_address(address)
	}

	pub fn from_address(address: Address) -> Result<Self, KeyError> {
		Ok(PublicKey {
			ed: OnceCell::new(),
			x: OnceCell::new(),
			address: address,
		})
	}

	pub fn ed(&self) -> &DalekEdPublicKey {
		self.ed.get_or_init(|| {
			DalekEdPublicKey::from_bytes(&self.address.to_arr())
				.expect("Failed to create DalekEdPublicKey")
		})
	}

	pub fn x(&self) -> &DalekXPublicKey {
		self.x.get_or_init(|| {
			DalekXPublicKey::from(self.ed().to_montgomery().to_bytes())
		})
	}
}

impl Byteable<KeyError> for PublicKey {
	fn to_bytes(&self) -> Bytes {
		Bytes::copy_from_slice(&self.address.to_arr())
	}

	fn from_bytes(bytes: &Bytes) -> Result<Self, KeyError> {
		PublicKey::from_arr(
			bytes
				.as_ref()
				.try_into()
				.map_err(|_| KeyError::InvalidPublicKey)?,
		)
	}
}

impl Stringable<KeyError> for PublicKey {
	fn to_string(&self) -> String {
		bys::to_base32(&self.to_bytes())
	}

	fn from_string(string: &str) -> Result<Self, KeyError> {
		let bytes =
			bys::from_base32(string).map_err(|_| KeyError::InvalidPublicKey)?;

		PublicKey::from_bytes(&bytes)
	}
}

impl Arrable<KeyArr, KeyError> for PublicKey {
	fn to_arr(&self) -> KeyArr {
		self.address.to_arr()
	}

	fn from_arr(arr: &KeyArr) -> Result<Self, KeyError> {
		PublicKey::new(arr.clone())
	}
}

impl Verifiable for PublicKey {
	fn verify(
		&self,
		message: &Bytes,
		signature: &Signature,
	) -> Result<(), KeyError> {
		self.ed()
			.verify(message.as_ref(), signature)
			.map_err(|_| KeyError::InvalidSignature)
	}
}

/**
 * PrivateKey
*/

impl PrivateKey {
	pub fn new(ed_arr: DalekEdPrivateKeyArr) -> Result<Self, KeyError> {
		let address = Address::from_arr(&ed_arr)
			.map_err(|_| KeyError::InvalidPrivateKey)?;

		Self::from_address(address)
	}

	pub fn from_address(address: Address) -> Result<Self, KeyError> {
		Ok(PrivateKey {
			ed: OnceCell::new(),
			x: OnceCell::new(),
			address,
		})
	}

	pub fn ed(&self) -> &DalekEdPrivateKey {
		self.ed.get_or_init(|| {
			DalekEdPrivateKey::from_bytes(&self.address.to_arr())
		})
	}

	pub fn x(&self) -> &DalekXPrivateKey {
		self.x
			.get_or_init(|| DalekXPrivateKey::from(self.ed().to_scalar_bytes()))
	}
}

impl Byteable<KeyError> for PrivateKey {
	fn to_bytes(&self) -> Bytes {
		Bytes::copy_from_slice(&self.address.to_arr())
	}

	fn from_bytes(bytes: &Bytes) -> Result<Self, KeyError> {
		PrivateKey::from_arr(
			bytes
				.as_ref()
				.try_into()
				.map_err(|_| KeyError::InvalidPrivateKey)?,
		)
	}
}

impl Stringable<KeyError> for PrivateKey {
	fn to_string(&self) -> String {
		bys::to_base32(&self.to_bytes())
	}

	fn from_string(string: &str) -> Result<Self, KeyError> {
		let bytes = bys::from_base32(string)
			.map_err(|_| KeyError::InvalidPrivateKey)?;

		PrivateKey::from_bytes(&bytes)
	}
}

impl Arrable<KeyArr, KeyError> for PrivateKey {
	fn to_arr(&self) -> KeyArr {
		self.address.to_arr()
	}

	fn from_arr(arr: &KeyArr) -> Result<Self, KeyError> {
		PrivateKey::new(arr.clone())
	}
}

impl Signable for PrivateKey {
	fn sign(&self, message: &Bytes) -> Signature {
		self.ed().sign(message.as_ref())
	}
}

impl SharedSecretable for PrivateKey {
	fn shared_secret(&self, public_key: &PublicKey) -> DalekXSharedSecret {
		self.x().diffie_hellman(&public_key.x())
	}
}

impl Randomable for PrivateKey {
	fn random() -> Self {
		PrivateKey::from_address(Address::random()).unwrap()
	}
}

/**
 * KeyPair
*/
impl KeyPair {
	pub fn new(private_key: PrivateKey) -> Result<Self, KeyError> {
		let public_key =
			PublicKey::new(private_key.ed().verifying_key().to_bytes())?;

		Ok(KeyPair {
			private_key,
			public_key,
		})
	}
}

impl Byteable<KeyError> for KeyPair {
	fn to_bytes(&self) -> Bytes {
		bys::concat(&[
			self.private_key().to_bytes(),
			self.public_key().to_bytes(),
		])
	}

	fn from_bytes(bytes: &Bytes) -> Result<Self, KeyError> {
		let private_key =
			PrivateKey::from_bytes(&bytes.slice(0..SECRET_KEY_LENGTH))?;

		KeyPair::new(private_key)
	}
}

impl Stringable<KeyError> for KeyPair {
	fn to_string(&self) -> String {
		bys::to_base32(&self.to_bytes())
	}

	fn from_string(string: &str) -> Result<Self, KeyError> {
		let bytes =
			bys::from_base32(string).map_err(|_| KeyError::InvalidKeyPair)?;

		KeyPair::from_bytes(&bytes)
	}
}

impl Signable for KeyPair {
	fn sign(&self, message: &Bytes) -> Signature {
		self.private_key().sign(message)
	}
}

impl Verifiable for KeyPair {
	fn verify(
		&self,
		message: &Bytes,
		signature: &Signature,
	) -> Result<(), KeyError> {
		self.public_key().verify(message, signature)
	}
}

impl SharedSecretable for KeyPair {
	fn shared_secret(&self, public_key: &PublicKey) -> DalekXSharedSecret {
		self.private_key().shared_secret(public_key)
	}
}

impl Randomable for KeyPair {
	fn random() -> Self {
		let private_key = PrivateKey::random();

		KeyPair::new(private_key).unwrap()
	}
}

#[test]
fn test_keys() -> Result<(), KeyError> {
	let alice = KeyPair::random();
	let bob = KeyPair::random();

	// alice signs a message
	let message = Bytes::from("Hello, world!");
	let signature = alice.sign(&message);

	// alice verifies the signature
	alice.verify(&message, &signature)?;

	// bob verifies the signature with alice's public key
	let alice_public_key =
		PublicKey::from_bytes(&alice.public_key().to_bytes())?;
	alice_public_key.verify(&message, &signature)?;

	// compute shared secret
	let alice_shared_secret = alice.shared_secret(&bob.public_key());
	let bob_shared_secret = bob.shared_secret(&alice.public_key());

	assert_eq!(alice_shared_secret.as_bytes(), bob_shared_secret.as_bytes());

	Ok(())
}
