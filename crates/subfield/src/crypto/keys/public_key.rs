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
#[wasm_bindgen]
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
 * Vecable
*/
impl Vecable<KeyError> for PublicKey {
	fn to_vec(&self) -> Vec<u8> {
		self.v256().to_vec()
	}
	
	fn from_arr(arr: &[u8]) -> Result<Self, KeyError> {
		Ok(PublicKey::new(V256::from_arr(arr).map_err(|_| KeyError::InvalidPublicKey)?))
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


#[wasm_bindgen]
impl PublicKey {
	
	/**
	 * Constructor
	*/
	#[wasm_bindgen(constructor)]
	pub fn _js_new(v256: V256) -> Self {
		PublicKey::new(v256)
	}
	
	/**
	 * Getters
	*/
	#[wasm_bindgen(getter, js_name = "ed")]
	pub fn _js_ed(&self) -> Uint8Array {
		self.ed().to_bytes().as_slice().into()
	}
	
	#[wasm_bindgen(getter, js_name = "x")]
	pub fn _js_x(&self) -> Uint8Array {
		self.x().to_bytes().as_slice().into()
	}
	
	/**
	 * Verify
	*/
	#[wasm_bindgen(js_name = "verify")]
	pub fn _js_verify(
		&self,
		message: &[u8],
		signature: &Signature,
	) -> bool {
		self.verify(message, signature).unwrap()
	}
	
	/**
	 * Byteable
	*/
	#[wasm_bindgen(js_name = "toBytes")]
	pub fn _js_to_bytes(&self) -> Uint8Array {
		self.v256()._js_to_bytes()
	}
	
	#[wasm_bindgen(js_name = "fromBytes")]
	pub fn _js_from_bytes(bytes: Uint8Array) -> Self {
		PublicKey::new(V256::_js_from_bytes(bytes))
	}
	
	/**
	 * Stringable
	*/
	#[wasm_bindgen(js_name = "toString")]
	pub fn _js_to_string(&self) -> String {
		self.v256()._js_to_string()
	}
	
	#[wasm_bindgen(js_name = "fromString")]
	pub fn _js_from_string(string: &str) -> Self {
		PublicKey::new(V256::_js_from_string(string))
	}
}