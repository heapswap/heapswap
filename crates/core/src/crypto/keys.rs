use std::convert::From;

use bytes::Bytes;
use derive_getters::Getters;
//use getset::Getters;
use derive_more::{Display, Error};
use rand::rngs::OsRng;
use rand::RngCore;
use timeit::*;

use ed25519_dalek::{
    Signature, Signer, SigningKey as DalekEdPrivateKey, Verifier, VerifyingKey as DalekEdPublicKey,
};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};
use x25519_dalek::{
    PublicKey as DalekXPublicKey, SharedSecret as DalekXSharedSecret, StaticSecret as DalekXPrivateKey,
};

use crate::bys;
use crate::comparison::{hamming, xor};
use crate::traits::*;

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

pub type EdPublicKey = U256;
pub type EdPrivateKey = U256;

pub type XPublicKey = U256;


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

#[derive(Getters)]
pub struct PrivateKey {
    ed: DalekEdPrivateKey,
    ed_u256: U256,
    x: DalekXPrivateKey,
}

#[derive(Getters)]
pub struct PublicKey {
    ed: DalekEdPublicKey,
    ed_u256: U256,
    x: DalekXPublicKey,
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
    fn shared_secret(&self, public_key: &PublicKey) -> DalekXSharedSecret;
}

/**
 * Implementations
*/

// PublicKey

impl PublicKey {
    pub fn new(ed_arr: DalekEdPublicKeyArr) -> Result<Self, KeyError> {
        let ed =
            DalekEdPublicKey::from_bytes(&ed_arr).map_err(|_| KeyError::FailedToCreateDalekEdPublicKey)?;

        let x = DalekXPublicKey::from(ed.to_montgomery().to_bytes());

        let ed_u256 = U256::from_arr(&ed_arr).map_err(|_| KeyError::InvalidPublicKey)?;

        Ok(PublicKey { ed, x, ed_u256 })
    }

    pub fn from_ed_u256(ed_u256: U256) -> Result<Self, KeyError> {
        let ed_arr = U256::to_arr(&ed_u256);

        let ed =
            DalekEdPublicKey::from_bytes(&ed_arr).map_err(|_| KeyError::FailedToCreateDalekEdPublicKey)?;

        let x = DalekXPublicKey::from(ed.to_montgomery().to_bytes());

        Ok(PublicKey { ed, x, ed_u256: ed_u256 })
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
    pub fn new(ed_arr: DalekEdPrivateKeyArr) -> Result<Self, KeyError> {
        let ed = DalekEdPrivateKey::from_bytes(&ed_arr);

        let x = DalekXPrivateKey::from(ed.to_scalar_bytes());

        Ok(PrivateKey {
            ed,
            x,
            ed_u256: U256::from_arr(&ed_arr).map_err(|_| KeyError::InvalidPrivateKey)?,
        })
    }

    pub fn from_ed_u256(ed_u256: U256) -> Result<Self, KeyError> {
        let ed_arr = ed_u256.to_arr();

        let ed = DalekEdPrivateKey::from_bytes(&ed_arr);

        let x = DalekXPrivateKey::from(ed.to_scalar_bytes());

        Ok(PrivateKey { ed, x, ed_u256 })
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
    fn shared_secret(&self, public_key: &PublicKey) -> DalekXSharedSecret {
        self.x().diffie_hellman(&public_key.x())
    }
}

impl Randomable<KeyError> for PrivateKey {
    fn random() -> Result<Self, KeyError> {
        let mut csprng = OsRng;
        let signing_key = DalekEdPrivateKey::generate(&mut csprng);

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
    fn shared_secret(&self, public_key: &PublicKey) -> DalekXSharedSecret {
        self.private_key().shared_secret(public_key)
    }
}

impl Randomable<KeyError> for KeyPair {
    fn random() -> Result<Self, KeyError> {
        let private_key = PrivateKey::random()?;

        KeyPair::new(private_key)
    }
}

#[test]
fn test_keys() -> Result<(), KeyError> {
    let alice = KeyPair::random()?;
    let bob = KeyPair::random()?;

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