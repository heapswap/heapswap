// SPDX-FileCopyrightText: 2023 Dominik George <nik@naturalnet.de>
//
// SPDX-License-Identifier: Apache-2.0

//! Generic interface for XEdDSA implementations

use derive_more::{Display, Error};
use zeroize::{Zeroize, ZeroizeOnDrop};
use bytes::Bytes;

use rand::{CryptoRng, RngCore};

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

/// Methods to calculate EdDSA key pairs from a Diffie-Hellman private key
pub trait CalculateKeyPair<PrivT: AsRef<[u8]>, PubT: AsRef<[u8]>>: Zeroize + ZeroizeOnDrop {
    /// Calculate an EdDSA private and public key pair
    ///
    /// This calculation yields a private key that can be used for XEdDSA
    /// signing, and a public key that can be used to verify the signature
    /// with any EdDSA implementation.
    fn calculate_key_pair(&self, sign: u8) -> (PrivT, PubT);
}

/// Methods to sign a message using XEdDSA
pub trait Sign<SigT, PrivT: AsRef<[u8]>, PubT: AsRef<[u8]>>: CalculateKeyPair<PrivT, PubT> {
    fn sign(&self, message: &[u8], rng: impl RngCore + CryptoRng) -> SigT;
}

/// Methods to calculate EdDSA public keys from Diffie-Hellman public keys
pub trait ConvertMont<PubT: AsRef<[u8]>> {
    /// Convert this key to an EdDSA public key
    fn convert_mont(&self, sign: u8) -> Result<PubT, Error>;
}

/// Methods to verify messages against XEdDSA signatures
pub trait Verify<SigT, PubT: AsRef<[u8]>>: ConvertMont<PubT> {
    /// Verify a message against a signature
    fn verify(&self, message: &[u8], signature: &SigT) -> Result<(), Error>;
}

