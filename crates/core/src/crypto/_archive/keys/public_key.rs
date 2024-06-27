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

use super::{EdPublicKeyBytes, Error, PublicKey, Sign, SignatureBytes, ToEdwards, Verify, XPublicKeyBytes};


impl ToEdwards<XPublicKeyBytes, EdPublicKeyBytes> for PublicKey{
	fn to_edwards(key: XPublicKeyBytes) -> Result<EdPublicKeyBytes, Error> {
		//Convert Montgomery point to Edwards point, forcing the sign
		let sign = 0;
		let edwards_point = MontgomeryPoint(key)
				.to_edwards(sign)
				.ok_or(Error::UnusablePublicKey).unwrap();
		Ok(edwards_point.compress().to_bytes())
	}
}

impl Verify for PublicKey{
	fn verify(&self, message: &[u8], signature: &SignatureBytes) -> Result<(), Error>  {
		let verifying_key =
		VerifyingKey::from_bytes(&self.ed()).or(Err(Error::UnusablePublicKey))?;
		verifying_key
			.verify(message, &Signature::from_bytes(signature))
			.or(Err(Error::InvalidSignature))
	}
}