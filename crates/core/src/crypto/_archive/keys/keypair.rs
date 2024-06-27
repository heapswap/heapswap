use std::convert::From;

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

use super::{EdPrivateKeyBytes, KeyPair, Sign, SignatureBytes, ToEdwards, XPrivateKeyBytes, Error};

const SIGNATURE_LENGTH: usize = 64;


impl Sign for KeyPair{
	fn sign(&self, message: &[u8]) -> SignatureBytes {
		
		let mut rng = rand::rngs::OsRng;
		
        // Derive EdDSA key pair
        //let (private_key, public_key) = self.calculate_key_pair(0);

        // Take 64 bytes random padding for hash function
        let mut nonce = [0u8; 64];
        rng.fill_bytes(&mut nonce);

        // Do hash_1 with SHA-512 over private key, message and nonce
        let padding: [u8; 32] = hash_i_padding(1);
        let mut hasher = Sha512::new();
        hasher.update(padding);
        hasher.update(self.private_key().ed());
        hasher.update(message);
        hasher.update(nonce);
        let res: [u8; 64] = hasher.finalize().into();

        // Calculate R = rB
        let res_scalar = Scalar::from_bytes_mod_order_wide(&res);
        let res_point = EdwardsPoint::mul_base(&res_scalar);

        // Do SHA-512 hash over R, public key and the message
        let mut hasher = Sha512::new();
        hasher.update(res_point.compress().to_bytes());
        hasher.update(self.public_key().ed());
        hasher.update(message);
        let hash: [u8; 64] = hasher.finalize().into();

        // Calculate s = r + ha
        // (All operations in curve25519_dalek are (mod q))
        let hash_scalar = Scalar::from_bytes_mod_order_wide(&hash);
        let private_scalar = Scalar::from_bytes_mod_order(self.private_key().ed().clone());
        let salt = res_scalar + hash_scalar * private_scalar;

        let mut signature = [0u8; SIGNATURE_LENGTH];
        signature[0..32].copy_from_slice(&res_point.compress().to_bytes());
        signature[32..64].copy_from_slice(&salt.to_bytes());

        signature
	}
}