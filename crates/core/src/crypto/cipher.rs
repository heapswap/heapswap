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
pub type Secret = U256; //[u8; 32];
pub type SecretArray = [u8; 32];
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
	secret: U256,
	cipher: ChaCha20Poly1305,
}

impl Cipher {
	/**
	 * Decrypt
		*/
	pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CipherError> {
		// check that the ciphertext is at least as long as the nonce
		if ciphertext.len() < NONCE_LENGTH {
			return Err(CipherError::InvalidNonce);
		}

		// Extract nonce from the ciphertext
		let nonce: NonceArray = ciphertext[0..NONCE_LENGTH]
			.try_into()
			.map_err(|_| CipherError::InvalidNonce)?;

		// Decryption
		Ok(self
			.cipher
			.decrypt(&nonce.into(), &ciphertext[NONCE_LENGTH..])
			.unwrap())
	}

	/**
	 * Encrypt
		*/
	pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
		// Generate nonce
		let nonce: NonceArray =
			ChaCha20Poly1305::generate_nonce(&mut OsRng).into();

		// Convert nonce to Uint8Array using Uint8Array::copy_from_slice
		let encrypted_data = self
			.cipher
			.encrypt(&nonce.into(), plaintext.to_vec().as_slice())
			.unwrap();

		// Concatenate nonce and encrypted data
		arr::concat(&[&nonce, encrypted_data.as_slice()])
	}
}

#[wasm_bindgen]
impl Cipher {
	/**
	 * Constructors
		*/

	#[wasm_bindgen(constructor)]
	pub fn new(secret: Secret) -> Cipher {
		let cipher = ChaCha20Poly1305::new(secret.unpacked().into());

		Cipher { secret, cipher }
	}

	/**
	 * Random
		*/

	#[wasm_bindgen(js_name = randomSecret)]
	pub fn random_secret() -> Secret {
		Secret::new(U256::random().unpacked()).unwrap()
	}

	#[wasm_bindgen(js_name = encrypt)]
	pub fn _js_encrypt(&self, plaintext: Plaintext) -> Ciphertext {
		Uint8Array::from(self.encrypt(plaintext.to_vec().as_slice()).as_slice())
	}

	#[wasm_bindgen(js_name = decrypt)]
	pub fn _js_decrypt(
		&self,
		ciphertext: Ciphertext,
	) -> Result<Plaintext, CipherError> {
		self.decrypt(ciphertext.to_vec().as_slice())
			.map(|plaintext| Uint8Array::from(plaintext.as_slice()))
	}
}
