use std::convert::From;

use curve25519_dalek::edwards::EdwardsPoint;
use curve25519_dalek::montgomery::MontgomeryPoint;
use curve25519_dalek::scalar::{clamp_integer, Scalar};
use ed25519_dalek::ed25519::signature::Keypair;
use ed25519_dalek::Signature;
use ed25519_dalek::{Verifier, VerifyingKey};
use x25519_dalek;

use rand::{CryptoRng, RngCore};
use sha2::{Digest, Sha512};
use derive_more::{Display, Error};
use zeroize::{Zeroize, ZeroizeOnDrop};
use bytes::Bytes;
use derive_getters::Getters;

use crate::traits::Randomable;

pub const PRIVATE_KEY_LENGTH: usize = 32;
pub const PUBLIC_KEY_LENGTH: usize = 32;
pub const SIGNATURE_LENGTH: usize = 64;

pub type KeyBytes = [u8; PRIVATE_KEY_LENGTH];

pub type PrivateKeyBytes = [u8; PRIVATE_KEY_LENGTH];
pub type XPrivateKeyBytes = [u8; PRIVATE_KEY_LENGTH];
pub type EdPrivateKeyBytes = [u8; PRIVATE_KEY_LENGTH];

pub type PublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];
pub type XPublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];
pub type EdPublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];

pub type SignatureBytes = [u8; SIGNATURE_LENGTH];



/// Error type for all XEdDSA operations
#[derive(Debug, Display, Error)]
pub enum Error {
    /// The signature to be verified was invalidk
    #[display(fmt = "Invalid signature")]
    InvalidSignature,

    /// The public key is not usable for verification (weak or otherwise unusable key)
    #[display(fmt = "Unusable or weak public key")]
    UnusablePublicKey,
}


#[derive(Zeroize, ZeroizeOnDrop, Getters)]
pub struct PrivateKey{
    x: XPrivateKeyBytes,
    ed: EdPrivateKeyBytes
}

impl PrivateKey{
    pub fn new(x: XPrivateKeyBytes) -> Self {
        PrivateKey{
            x,
            ed: PrivateKey::to_edwards(x).unwrap()
        }
    }
}



#[derive(Getters)]
pub struct PublicKey{
    x: XPublicKeyBytes,
    ed: EdPublicKeyBytes
}


impl PublicKey{
    pub fn new(x: XPublicKeyBytes) -> Self {
        PublicKey{
            x,
            ed: PublicKey::to_edwards(x).unwrap()
        }
    }
}


#[derive(Getters)]
pub struct KeyPair{
	public_key: PublicKey,
	private_key: PrivateKey
}

impl KeyPair{
    pub fn new(private_key: PrivateKey) -> Self {
        KeyPair{
            private_key,
            public_key: PrivateKey::
        }
    }
}

pub trait ToEdwards<F,T>{
	fn to_edwards(&self) -> Result<T, Error>;
}

pub trait Sign{
	fn sign(&self, message: &[u8]) -> SignatureBytes;
}

pub trait Verify{
	fn verify(&self, message: &[u8], signature: &SignatureBytes) -> Result<(), Error> ;
}

pub trait SharedSecret{
	fn shared_secret(&self, public_key: &PublicKey) -> KeyBytes;
}


impl Randomable for PrivateKey{
	fn from_random() -> Self {
		let mut rng = rand::rngs::OsRng;
		let mut private_key = [0u8; 32];
		rng.fill_bytes(&mut private_key);
		
		PrivateKey{
			x: private_key,
			ed: PrivateKey::to_edwards(private_key).unwrap()
		}
	}
}

//impl Randomable for Keypair