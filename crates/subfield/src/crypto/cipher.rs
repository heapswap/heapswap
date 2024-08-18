use crate::*;
use crate::*;
use chacha20poly1305::{
	aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, OsRng},
	ChaCha20Poly1305,
};

pub type Plaintext = Vec<u8>;
pub type Ciphertext = Vec<u8>;
pub type SecretKey = V256;
pub type SecretKeyArray = [u8; 32];
const NONCE_LENGTH: usize = 12;
pub type Nonce = Vec<u8>; // [u8; NONCE_LENGTH];
pub type NonceArray = [u8; NONCE_LENGTH];

#[derive(Debug)]
pub enum CipherError {
	InvalidNonce,
	InvalidKey,
}

pub struct Cipher {
	secret: V256,
	cipher: ChaCha20Poly1305,
}

impl Cipher {
	/**
	 * Constructors
		*/

	pub fn new(secret: SecretKey) -> Cipher {
		let cipher =
			ChaCha20Poly1305::new(&GenericArray::from(secret.bytes().clone()));

		Cipher { secret, cipher }
	}

	pub fn random() -> Cipher {
		Cipher::new(V256::random())
	}

	/**
	 * Getters
		*/
	pub fn secret(&self) -> &SecretKey {
		&self.secret
	}

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

		// Convert nonce to Vec<u8> using Vec<u8>::copy_from_slice
		let encrypted_data = self
			.cipher
			.encrypt(&nonce.into(), plaintext.to_vec().as_slice())
			.unwrap();

		// Concatenate nonce and encrypted data
		[&nonce, encrypted_data.as_slice()].concat()
	}
}

#[test]
fn test_cipher() {
	let cipher = Cipher::random();
	let plaintext = b"hello world";
	let ciphertext = cipher.encrypt(plaintext);
	let decrypted = cipher.decrypt(&ciphertext).unwrap();
	assert_eq!(plaintext.to_vec(), decrypted);
}
