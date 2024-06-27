use std::convert::From;

use crate::traits::Randomable;

use super::utils::hash_i_padding;

use curve25519_dalek::edwards::EdwardsPoint;
use curve25519_dalek::montgomery::MontgomeryPoint;
use curve25519_dalek::scalar::{clamp_integer, Scalar};
use ed25519_dalek::Signature;
use ed25519_dalek::{Verifier, VerifyingKey};
use x25519_dalek;

use rand::{CryptoRng, RngCore};
use sha2::{Digest, Sha512};

use zeroize::{Zeroize, ZeroizeOnDrop};

use super::{EdPrivateKeyBytes, Error, PrivateKey, PublicKey, Sign, SignatureBytes, ToEdwards, XPrivateKeyBytes};

impl ToEdwards<XPrivateKeyBytes, EdPrivateKeyBytes> for PrivateKey{
	fn to_edwards(&self) -> Result<EdPrivateKeyBytes, Error> {
		
		// Clamp the (scalar) private key to be within the curve field
		let clamped = clamp_integer(self.x().clone());

		// Derive Ed25519 public key to verify sign
		let scalar_private_key = Scalar::from_bytes_mod_order(clamped);
		
		let point_public_key = EdwardsPoint::mul_base(&scalar_private_key);
		
		let sign = 0;
		if (point_public_key.compress().to_bytes()[31] & 0x80) >> 7 == sign {
			// Sign matches, return verbatim
			return Ok(clamped)
		} else {
			
			// Negate the scalar and calculate new key
			let scalar_private_key = (Scalar::ZERO - Scalar::from(1_u8)) * scalar_private_key;
			
			return Ok(scalar_private_key.to_bytes())
		}
	}
}


trait ToPublicKey{
	fn to_public_key(&self) -> PublicKey;
}

impl ToPublicKey for PrivateKey{
	fn to_public_key(&self) -> PublicKey{
		// Clamp the (scalar) private key to be within the curve field
		let clamped = clamp_integer(self.x().clone());

		// Derive Ed25519 public key to verify sign
		let scalar_private_key = Scalar::from_bytes_mod_order(clamped);
				
		let point_public_key = EdwardsPoint::mul_base(&scalar_private_key).compress().to_bytes();
		
		return 
				
	}
}