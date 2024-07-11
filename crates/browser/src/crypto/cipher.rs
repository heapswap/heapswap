use crate::arr;
use crate::u256::*;
use chacha20poly1305::{
	aead::{Aead, AeadCore, KeyInit, OsRng},
	ChaCha20Poly1305,
};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

pub type Plaintext = Uint8Array;
pub type Ciphertext = Uint8Array;
pub type SharedKey = U256; //[u8; 32];
pub type SharedKeyArray = [u8; 32];
const NONCE_LENGTH: usize = 12;
pub type Nonce = Uint8Array; // [u8; NONCE_LENGTH];
pub type NonceArray = [u8; NONCE_LENGTH];

#[wasm_bindgen]
#[derive(Debug)]
pub enum CipherError {
	InvalidNonce,
	InvalidKey,
}

#[wasm_bindgen]
pub struct Cipher {
	cipher: ChaCha20Poly1305,
}

#[wasm_bindgen]
impl Cipher {
	/**
	 * Constructors
		*/

	#[wasm_bindgen(constructor)]
	pub fn new(key: SharedKey) -> Result<Cipher, CipherError> {
		//let shared_key_array: SharedKeyArray = key.to_vec().try_into().map_err(|_| CipherError::InvalidKey)?;
		//Ok(Cipher::from_array(shared_key_array))

		Ok(Self::from_array(key.unpacked().clone()))
	}

	fn from_array(shared_key_array: SharedKeyArray) -> Self {
		Cipher {
			cipher: ChaCha20Poly1305::new(&shared_key_array.into()),
		}
	}

	/**
	 * Randomable
		*/

	#[wasm_bindgen(js_name = randomSecret)]
	pub fn random_secret() -> SharedKey {
		SharedKey::new(U256::random().unpacked()).unwrap()
	}

	/**
	 * Encryptable
		*/

	#[wasm_bindgen]
	pub fn encrypt(&self, plaintext: Plaintext) -> Ciphertext {
		// Generate nonce
		let nonce: NonceArray =
			ChaCha20Poly1305::generate_nonce(&mut OsRng).into();

		// Convert nonce to Uint8Array using Uint8Array::copy_from_slice
		let encrypted_data = self
			.cipher
			.encrypt(&nonce.into(), plaintext.to_vec().as_slice())
			.unwrap();

		// Concatenate nonce and encrypted data
		let result = arr::concat(&[&nonce, encrypted_data.as_slice()]);

		// Convert result to Uint8Array
		Uint8Array::from(result.as_slice())
	}

	#[wasm_bindgen]
	pub fn decrypt(
		&self,
		ciphertext: Ciphertext,
	) -> Result<Plaintext, CipherError> {
		let ciphertext = ciphertext.to_vec();

		// check that the ciphertext is at least as long as the nonce
		if ciphertext.len() < NONCE_LENGTH {
			return Err(CipherError::InvalidNonce);
		}

		// Extract nonce from the ciphertext
		let nonce: NonceArray = ciphertext[0..NONCE_LENGTH]
			.try_into()
			.map_err(|_| CipherError::InvalidNonce)?;

		// Actual decryption
		let decrypted_data = self
			.cipher
			.decrypt(&nonce.into(), &ciphertext[NONCE_LENGTH..])
			.unwrap();

		// Convert decrypted data to Uint8Array
		Ok(Uint8Array::from(decrypted_data.as_slice()))
	}
}
