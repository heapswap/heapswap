use std::convert::From;

use bytes::Bytes;
use derive_getters::Getters;
use derive_more::{Display, Error};
use rand::rngs::OsRng;
use timeit::*;

use ed25519_dalek::{
    Signature, Signer, SigningKey as EdPrivateKey, Verifier, VerifyingKey as EdPublicKey,
};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};
use x25519_dalek::{
    PublicKey as XPublicKey, SharedSecret as XSharedSecret, StaticSecret as XPrivateKey,
};

use crate::bys;
use crate::comparison::{hamming, xor};
use crate::traits::{Byteable, Randomable, Stringable};

use super::utils::{stack_256, unstack_256};

/**
 * Types
*/
pub type KeyBytes = [u8; SECRET_KEY_LENGTH];

pub type PrivateKeyBytes = [u8; SECRET_KEY_LENGTH];
pub type XPrivateKeyBytes = [u8; SECRET_KEY_LENGTH];
pub type EdPrivateKeyBytes = [u8; SECRET_KEY_LENGTH];

pub type PublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];
pub type XPublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];
pub type EdPublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];

pub type SignatureBytes = [u8; SIGNATURE_LENGTH];

pub type StackedKey = [u64; 4]; // to be able to do xor more efficiently
pub type Extended = [u64; 16]; // for future use

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
    FailedToCreateEdPublicKey,
    FailedToCreateEdPrivateKey,
}

/**
 * Structs
*/

#[derive(Getters)]
pub struct PrivateKey {
    ed: EdPrivateKey,
    x: XPrivateKey,
    stacked: StackedKey,
}

#[derive(Getters)]
pub struct PublicKey {
    ed: EdPublicKey,
    x: XPublicKey,
    stacked: StackedKey,
}

#[derive(Getters)]
pub struct KeyPair {
    public_key: PublicKey,
    private_key: PrivateKey,
}

/**
 * Traits
*/

pub trait Signable {
    fn sign(&self, message: &Bytes) -> Signature;
}

pub trait Verifiable {
    fn verify(&self, message: &Bytes, signature: &Signature) -> Result<(), KeyError>;
}

pub trait SharedSecretable {
    fn shared_secret(&self, public_key: &PublicKey) -> XSharedSecret;
}

/**
 * Implementations
*/

// PublicKey

impl PublicKey {
    pub fn new(ed_bytes: EdPublicKeyBytes) -> Result<Self, KeyError> {
        let ed =
            EdPublicKey::from_bytes(&ed_bytes).map_err(|_| KeyError::FailedToCreateEdPublicKey)?;

        let x = XPublicKey::from(ed.to_montgomery().to_bytes());

        let stacked = stack_256(&ed_bytes);

        Ok(PublicKey { ed, x, stacked })
    }

    pub fn from_stacked(stacked: &StackedKey) -> Result<Self, KeyError> {
        let ed_bytes = unstack_256(stacked);

        let ed =
            EdPublicKey::from_bytes(&ed_bytes).map_err(|_| KeyError::FailedToCreateEdPublicKey)?;

        let x = XPublicKey::from(ed.to_montgomery().to_bytes());

        Ok(PublicKey {
            ed,
            x,
            stacked: stacked.clone(),
        })
    }
}

impl Byteable<KeyError> for PublicKey {
    fn to_bytes(&self) -> Bytes {
        Bytes::copy_from_slice(&self.ed().to_bytes())
    }

    fn from_bytes(bytes: &Bytes) -> Result<Self, KeyError> {
        PublicKey::new(bytes.as_ref().try_into().unwrap())
    }
}

impl Stringable<KeyError> for PublicKey {
    fn to_string(&self) -> String {
        bys::to_base32(&self.to_bytes())
    }

    fn from_string(string: &str) -> Result<Self, KeyError> {
        let bytes = bys::from_base32(string).map_err(|_| KeyError::InvalidPublicKey)?;

        PublicKey::from_bytes(&bytes)
    }
}

impl Verifiable for PublicKey {
    fn verify(&self, message: &Bytes, signature: &Signature) -> Result<(), KeyError> {
        self.ed()
            .verify(message.as_ref(), signature)
            .map_err(|_| KeyError::InvalidSignature)
    }
}

// PrivateKey

impl PrivateKey {
    pub fn new(ed_bytes: EdPrivateKeyBytes) -> Result<Self, KeyError> {
        let ed = EdPrivateKey::from_bytes(&ed_bytes);

        let x = XPrivateKey::from(ed.to_scalar_bytes());

        let stacked = stack_256(&ed_bytes);

        Ok(PrivateKey { ed, x, stacked })
    }

    pub fn from_stacked(stacked: &StackedKey) -> Result<Self, KeyError> {
        let ed_bytes = unstack_256(stacked);

        let ed = EdPrivateKey::from_bytes(&ed_bytes);

        let x = XPrivateKey::from(ed.to_scalar_bytes());

        Ok(PrivateKey {
            ed,
            x,
            stacked: stacked.clone(),
        })
    }
}

impl Byteable<KeyError> for PrivateKey {
    fn to_bytes(&self) -> Bytes {
        Bytes::copy_from_slice(&self.ed().to_bytes())
    }

    fn from_bytes(bytes: &Bytes) -> Result<Self, KeyError> {
        PrivateKey::new(bytes.as_ref().try_into().unwrap())
    }
}

impl Stringable<KeyError> for PrivateKey {
    fn to_string(&self) -> String {
        bys::to_base32(&self.to_bytes())
    }

    fn from_string(string: &str) -> Result<Self, KeyError> {
        let bytes = bys::from_base32(string).map_err(|_| KeyError::InvalidPrivateKey)?;

        PrivateKey::from_bytes(&bytes)
    }
}

impl Signable for PrivateKey {
    fn sign(&self, message: &Bytes) -> Signature {
        self.ed().sign(message.as_ref())
    }
}

impl SharedSecretable for PrivateKey {
    fn shared_secret(&self, public_key: &PublicKey) -> XSharedSecret {
        self.x().diffie_hellman(&public_key.x())
    }
}

impl Randomable<KeyError> for PrivateKey {
    fn from_random() -> Result<Self, KeyError> {
        let mut csprng = OsRng;
        let signing_key = EdPrivateKey::generate(&mut csprng);

        PrivateKey::new(signing_key.to_bytes())
    }
}

// KeyPair

impl KeyPair {
    pub fn new(private_key: PrivateKey) -> Result<Self, KeyError> {
        let public_key = PublicKey::new(private_key.ed().verifying_key().to_bytes())?;

        Ok(KeyPair {
            private_key,
            public_key,
        })
    }
}

impl Byteable<KeyError> for KeyPair {
    fn to_bytes(&self) -> Bytes {
        bys::concat(&[self.private_key().to_bytes(), self.public_key().to_bytes()])
    }

    fn from_bytes(bytes: &Bytes) -> Result<Self, KeyError> {
        let private_key = PrivateKey::from_bytes(&bytes.slice(0..SECRET_KEY_LENGTH))?;

        KeyPair::new(private_key)
    }
}

impl Stringable<KeyError> for KeyPair {
    fn to_string(&self) -> String {
        bys::to_base32(&self.to_bytes())
    }

    fn from_string(string: &str) -> Result<Self, KeyError> {
        let bytes = bys::from_base32(string).map_err(|_| KeyError::InvalidKeyPair)?;

        KeyPair::from_bytes(&bytes)
    }
}

impl Signable for KeyPair {
    fn sign(&self, message: &Bytes) -> Signature {
        self.private_key().sign(message)
    }
}

impl Verifiable for KeyPair {
    fn verify(&self, message: &Bytes, signature: &Signature) -> Result<(), KeyError> {
        self.public_key().verify(message, signature)
    }
}

impl SharedSecretable for KeyPair {
    fn shared_secret(&self, public_key: &PublicKey) -> XSharedSecret {
        self.private_key().shared_secret(public_key)
    }
}

impl Randomable<KeyError> for KeyPair {
    fn from_random() -> Result<Self, KeyError> {
        let private_key = PrivateKey::from_random()?;

        KeyPair::new(private_key)
    }
}

#[test]
fn test_keys() -> Result<(), KeyError> {
    let alice = KeyPair::from_random()?;
    let bob = KeyPair::from_random()?;

    // alice signs a message
    let message = Bytes::from("Hello, world!");
    let signature = alice.sign(&message);

    // alice verifies the signature
    alice.verify(&message, &signature)?;

    // bob verifies the signature with alice's public key
    let alice_public_key = PublicKey::from_bytes(&alice.public_key().to_bytes())?;
    alice_public_key.verify(&message, &signature)?;

    // compute shared secret
    let alice_shared_secret = alice.shared_secret(&bob.public_key());
    let bob_shared_secret = bob.shared_secret(&alice.public_key());

    assert_eq!(alice_shared_secret.as_bytes(), bob_shared_secret.as_bytes());

    Ok(())
}

//#[test]
fn time_key_comparisons() -> Result<(), KeyError> {
    let alice = KeyPair::from_random()?;
    let bob = KeyPair::from_random()?;

    // timings
    let alice_bytes: &[u8; 32] = &alice.public_key().to_bytes().as_ref().try_into().unwrap();
    let bob_bytes: &[u8; 32] = &bob.public_key().to_bytes().as_ref().try_into().unwrap();
    let alice_stacked = alice.public_key().stacked();
    let bob_stacked = bob.public_key().stacked();

    let xor_ms = timeit_loops!(1000, {
        let _ = xor(alice_bytes, bob_bytes);
    });

    let ham_ms = timeit_loops!(1000, {
        let _ = hamming(alice_bytes, bob_bytes);
    });

    println!("xor_u8_32: {}ns/loop", xor_ms * 100000000.); // 24 ns
    println!("ham_u8_32: {}ns/loop", ham_ms * 100000000.); // 33 ns

    let xor_ms = timeit_loops!(1000, {
        let _ = xor(alice_stacked, bob_stacked);
    });

    let ham_ms = timeit_loops!(1000, {
        let _ = hamming(alice_stacked, bob_stacked);
    });

    println!("xor_u64_4: {}ns/loop", xor_ms * 100000000.); // 4 ns (6x faster)
    println!("ham_u64_4: {}ns/loop", ham_ms * 100000000.); // 5 ns (6x faster)

    Ok(())
}
