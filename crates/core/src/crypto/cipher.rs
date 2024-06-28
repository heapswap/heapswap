use bytes::Bytes;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305,
};

use crate::bys;

pub type Plaintext = Bytes;
pub type Ciphertext = Bytes;
pub type SharedKey = [u8; 32];
const NONCE_LENGTH: usize = 12;
pub type Nonce = [u8; NONCE_LENGTH];

pub struct Cipher {
    cipher: ChaCha20Poly1305,
}

pub trait Ciphering {
    fn new(key: SharedKey) -> Self;
    fn random() -> Self;

    fn encrypt(&self, plaintext: Plaintext) -> Ciphertext;
    fn decrypt(&self, ciphertext: Ciphertext) -> Plaintext;
}

impl Ciphering for Cipher {
    fn new(key: SharedKey) -> Self {
        Cipher {
            cipher: ChaCha20Poly1305::new(&key.into()),
        }
    }

    fn random() -> Self {
        let key: SharedKey = ChaCha20Poly1305::generate_key(&mut OsRng).into();
        Cipher::new(key)
    }

    fn encrypt(&self, plaintext: Plaintext) -> Ciphertext {
        let nonce: Nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng).into();

        // Convert nonce to Bytes using Bytes::copy_from_slice
        let nonce_bytes = Bytes::copy_from_slice(&nonce);
        let encrypted_data = self
            .cipher
            .encrypt(&nonce.into(), plaintext.as_ref())
            .unwrap();
        let encrypted_data_bytes = Bytes::from(encrypted_data);

        bys::concat(&[nonce_bytes, encrypted_data_bytes])
    }

    fn decrypt(&self, ciphertext: Ciphertext) -> Plaintext {
        // Extract nonce from the ciphertext
        let nonce = &ciphertext[0..NONCE_LENGTH];
        let nonce = Nonce::try_from(nonce).expect("Failed to extract nonce");

        // Actual decryption
        let decrypted_data = self
            .cipher
            .decrypt(&nonce.into(), &ciphertext[NONCE_LENGTH..])
            .unwrap();
        Bytes::from(decrypted_data)
    }
}

#[test]
fn test_cipher() {
    let cipher = Cipher::random();
    let plaintext = Bytes::from("plaintext message");
    let ciphertext = cipher.encrypt(plaintext.clone());
    let decrypted = cipher.decrypt(ciphertext.clone());
    assert_eq!(plaintext, decrypted);
}
