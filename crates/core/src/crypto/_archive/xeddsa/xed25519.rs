// SPDX-FileCopyrightText: 2023 Dominik George <nik@naturalnet.de>
//
// SPDX-License-Identifier: Apache-2.x

//! Concrete XEdDSA implementation for X25519/Ed25519

use std::convert::From;

use super::util::hash_i_padding;
use crate::crypto::xeddsa::*;

use curve25519_dalek::edwards::EdwardsPoint;
use curve25519_dalek::montgomery::MontgomeryPoint;
use curve25519_dalek::scalar::{clamp_integer, Scalar};
use ed25519_dalek::Signature;
use ed25519_dalek::{Verifier, VerifyingKey};
use x25519_dalek;

use rand::{CryptoRng, RngCore};
use sha2::{Digest, Sha512};

use zeroize::{Zeroize, ZeroizeOnDrop};

const PRIVATE_KEY_LENGTH: usize = 32;
const PUBLIC_KEY_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = 64;

type XPrivateKeyBytes = [u8; PRIVATE_KEY_LENGTH];
type XPublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];
type EdPrivateKeyBytes = [u8; PRIVATE_KEY_LENGTH];
type EdPublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];

type SignatureBytes = [u8; SIGNATURE_LENGTH];


/// An X25519 public key
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct PrivateKey{
    x: XPrivateKeyBytes,
    ed: EdPrivateKeyBytes
}

/// An X25519 public key
pub struct PublicKey{
    x: XPublicKeyBytes,
    ed: EdPublicKeyBytes
}


impl CalculateKeyPair<XPrivateKeyBytes, XPublicKeyBytes> for PrivateKey {
    fn calculate_key_pair(&self, sign: u8) -> (EdPrivateKeyBytes, EdPublicKeyBytes) {
        
        
        // Clamp the (scalar) private key to be within the curve field
        let clamped = clamp_integer(self.x);

        // Derive Ed25519 public key to verify sign
        let scalar_private_key = Scalar::from_bytes_mod_order(clamped);
        
        let point_public_key = EdwardsPoint::mul_base(&scalar_private_key);
        
        if (point_public_key.compress().to_bytes()[31] & 0x80) >> 7 == sign {
            // Sign matches, return verbatim
            (clamped, point_public_key.compress().to_bytes())
        } else {
            // Negate the scalar and calculate new key pair
            let scalar_private_key = (Scalar::ZERO - Scalar::from(1_u8)) * scalar_private_key;
            let point_public_key = EdwardsPoint::mul_base(&scalar_private_key);
            (
                scalar_private_key.to_bytes(),
                point_public_key.compress().to_bytes(),
            )
        }
    }
    
    fn calculate_ed_private_key(&self) -> EdPrivateKeyBytes {
        let clamped = clamp_integer(self.x);
        clamped
    }
    
    
    
}

impl Sign<SignatureBytes, XPrivateKeyBytes, XPublicKeyBytes> for PrivateKey
where
    PrivateKey: CalculateKeyPair<XPrivateKeyBytes, XPublicKeyBytes>,
{
    fn sign(&mut self, message: &[u8], mut rng: impl RngCore + CryptoRng) -> SignatureBytes {
        
        let (private_key, public_key) = self.calculate_key_pair(0);
        
        // Derive EdDSA key pair
        //let (private_key, public_key) = self.calculate_key_pair(0);

        // Take 64 bytes random padding for hash function
        let mut nonce = [0u8; 64];
        rng.fill_bytes(&mut nonce);

        // Do hash_1 with SHA-512 over private key, message and nonce
        let padding: [u8; 32] = hash_i_padding(1);
        let mut hasher = Sha512::new();
        hasher.update(padding);
        hasher.update(private_key);
        hasher.update(message);
        hasher.update(nonce);
        let res: [u8; 64] = hasher.finalize().into();

        // Calculate R = rB
        let res_scalar = Scalar::from_bytes_mod_order_wide(&res);
        let res_point = EdwardsPoint::mul_base(&res_scalar);

        // Do SHA-512 hash over R, public key and the message
        let mut hasher = Sha512::new();
        hasher.update(res_point.compress().to_bytes());
        hasher.update(public_key);
        hasher.update(message);
        let hash: [u8; 64] = hasher.finalize().into();

        // Calculate s = r + ha
        // (All operations in curve25519_dalek are (mod q))
        let hash_scalar = Scalar::from_bytes_mod_order_wide(&hash);
        let private_scalar = Scalar::from_bytes_mod_order(private_key);
        let salt = res_scalar + hash_scalar * private_scalar;

        let mut signature = [0u8; SIGNATURE_LENGTH];
        signature[0..32].copy_from_slice(&res_point.compress().to_bytes());
        signature[32..64].copy_from_slice(&salt.to_bytes());

        signature
    }
}

impl Sign<Signature, XPrivateKeyBytes, XPublicKeyBytes> for PrivateKey
where
    PrivateKey: CalculateKeyPair<XPrivateKeyBytes, XPublicKeyBytes>,
{
    fn sign(&mut self, message: &[u8], rng: impl RngCore + CryptoRng) -> Signature {
        Signature::from_bytes(&self.sign(message, rng))
    }
}

impl From<&XPrivateKeyBytes> for PrivateKey {
    fn from(value: &XPrivateKeyBytes) -> PrivateKey {
        let mut value_c: XPrivateKeyBytes = [0u8; PRIVATE_KEY_LENGTH];
        value_c.copy_from_slice(value);
        PrivateKey{
            x: value_c,
            ed: None
        }
    }
}

impl From<&x25519_dalek::StaticSecret> for PrivateKey {
    fn from(value: &x25519_dalek::StaticSecret) -> PrivateKey {
        PrivateKey::from(&value.to_bytes())
    }
}

impl From<PrivateKey> for XPrivateKeyBytes {
    fn from(value: PrivateKey) -> XPrivateKeyBytes {
        value.x
    }
}

impl ConvertMont<XPublicKeyBytes> for PublicKey {
    fn convert_mont(&self, sign: u8) -> Result<XPublicKeyBytes, Error> {
        //Convert Montgomery point to Edwards point, forcing the sign
        let edwards_point = MontgomeryPoint(self.x)
            .to_edwards(sign)
            .ok_or(Error::UnusablePublicKey)?;
        Ok(edwards_point.compress().to_bytes())
    }
}

impl Verify<Signature, XPublicKeyBytes> for PublicKey
where
    PublicKey: ConvertMont<XPublicKeyBytes>,
{
    fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), Error> {
        // Get EdDSA public key and verify using standard Ed25519 implementation
        let public_key = self.convert_mont(0)?;
        let verifying_key =
            VerifyingKey::from_bytes(&public_key).or(Err(Error::UnusablePublicKey))?;
        verifying_key
            .verify(message, signature)
            .or(Err(Error::InvalidSignature))
    }
}

impl Verify<SignatureBytes, XPublicKeyBytes> for PublicKey
where
    PublicKey: ConvertMont<XPublicKeyBytes>,
{
    fn verify(&self, message: &[u8], signature: &SignatureBytes) -> Result<(), Error> {
        self.verify(message, &Signature::from_bytes(signature))
    }
}

impl From<&x25519_dalek::PublicKey> for PublicKey {
    fn from(value: &x25519_dalek::PublicKey) -> PublicKey {
        PublicKey{
            x: value.to_bytes(),
            ed: None
        }
    }
}
