use crate::*;
use std::convert::From;
use std::iter::Once;
//use bytes::Bytes;
//use crypto_bigint::{Encoding, Random, Uint8Array};
//use derive_more::{Display, Error};
use getset::{CopyGetters, Getters, MutGetters, Setters};

use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use subfield_proto::versioned_bytes::VersionedBytes;

use crate::arr::{hamming, xor};
use crate::traits::*;
use ed25519_dalek::{
	Signature as DalekSignature, Signer, SigningKey as DalekEdPrivateKey,
	Verifier, VerifyingKey as DalekEdPublicKey,
};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};
use once_cell::sync::OnceCell;
use x25519_dalek::{
	PublicKey as DalekXPublicKey, SharedSecret as DalekXSharedSecret,
	StaticSecret as DalekXPrivateKey,
};

use super::private_key::*;
use super::public_key::*;
use super::{common::*, private_key};
use crate::arr;
use crate::versioned_bytes::*;
use subfield_proto::crypto::Keypair as ProtoKeypair;

#[derive(Clone, Getters)]
pub struct Keypair {
	proto: ProtoKeypair,
	#[getset(get = "pub")]
	private_key: PrivateKey,
	#[getset(get = "pub")]
	public_key: PublicKey,
}

impl Keypair {
	/**
	 * Constructors
		*/
	pub fn new(private_key: PrivateKey) -> Keypair {
		let public_key = private_key.public_key();

		Keypair {
			proto: ProtoKeypair {
				private_key: Some(private_key.versioned_bytes().clone()),
				public_key: Some(public_key.versioned_bytes().clone()),
			},
			private_key,
			public_key,
		}
	}

	pub fn from_proto(
		proto_keypair: ProtoKeypair,
	) -> Result<Keypair, KeyError> {
		Ok(Keypair {
			proto: proto_keypair.clone(),
			private_key: PrivateKey::new(
				proto_keypair
					.private_key
					.ok_or(KeyError::InvalidPrivateKey)?,
			),
			public_key: PublicKey::new(
				proto_keypair.public_key.ok_or(KeyError::InvalidPublicKey)?,
			),
		})
	}

	pub fn proto(&self) -> &ProtoKeypair {
		&self.proto
	}

	pub fn random() -> Self {
		let private_key = PrivateKey::random();
		Keypair::new(private_key)
	}

	pub fn vanity(prefix: &str) -> Result<Keypair, KeyError> {
		// test if prefix is valid base32
		let _ = arr::from_base32(prefix)
			.map_err(|_| KeyError::InvalidVanityPrefix)?;

		let mut counter = 0;
		loop {
			counter += 1;

			let private_key = DalekEdPrivateKey::generate(&mut OsRng);

			let public_key_string =
				arr::to_base32(&private_key.verifying_key().to_bytes());

			if public_key_string.starts_with(prefix) {
				return Ok(Keypair::new(PrivateKey::from_u256(
					private_key.to_bytes(),
				)));
			}
		}
	}

	/**
	 * Operations
		*/

	pub fn sign(&self, message: &[u8]) -> Signature {
		self.private_key().sign(message)
	}

	pub fn verify(
		&self,
		message: &[u8],
		signature: &Signature,
	) -> Result<bool, KeyError> {
		self.public_key().verify(message, signature)
	}

	pub fn shared_secret(&self, public_key: &PublicKey) -> SharedSecret {
		self.private_key().shared_secret(public_key)
	}
}

/**
 * Stringable
*/
impl Stringable<KeyError> for Keypair {
	fn to_string(&self) -> String {
		format!(
			"{}::{}",
			self.proto.private_key.as_ref().unwrap().to_string(),
			self.proto.public_key.as_ref().unwrap().to_string()
		)
	}

	fn from_string(string: &str) -> Result<Keypair, KeyError> {
		let parts = string.split("::").collect::<Vec<&str>>();
		Ok(Keypair::from_proto(ProtoKeypair {
			private_key: Some(
				VersionedBytes::from_string(parts[0])
					.map_err(|_| KeyError::InvalidPrivateKey)?,
			),
			public_key: Some(
				VersionedBytes::from_string(parts[1])
					.map_err(|_| KeyError::InvalidPublicKey)?,
			),
		})
		.map_err(|_| KeyError::InvalidKeypair)?)
	}
}

/**
 * Byteable
*/
impl Byteable<KeyError> for Keypair {
	fn to_bytes(&self) -> Bytes {
		Bytes::from(arr::from_proto::<ProtoKeypair>(&self.proto))
	}

	fn from_bytes(bytes: Bytes) -> Result<Keypair, KeyError> {
		Keypair::from_proto(
			ProtoKeypair::decode(&mut bytes.as_ref())
				.map_err(|_| KeyError::InvalidKeypair)?,
		)
	}
}

/**
 * Libp2pKeypairable
*/
impl Libp2pKeypairable<KeyError> for Keypair {
	fn to_libp2p_keypair(&self) -> libp2p::identity::Keypair {
		self.private_key().to_libp2p_keypair()
	}

	fn from_libp2p_keypair(
		libp2p_keypair: libp2p::identity::Keypair,
	) -> Result<Keypair, KeyError> {
		PrivateKey::from_libp2p_keypair(libp2p_keypair)
			.map(|private_key| Keypair::new(private_key))
	}
}

#[test]
fn test_sign_and_verify() {
	let keypair = Keypair::random();
	let message = b"hello world";
	let signature = keypair.sign(message);
	assert!(keypair.public_key().verify(message, &signature).unwrap());

	for i in 0..100 {
		let keypair = Keypair::random();
		println!("{}", keypair.private_key().to_string());
	}
}

#[test]
fn test_shared_secret() {
	let alice = Keypair::random();
	let bob = Keypair::random();
	let alice_shared_secret = alice.shared_secret(&bob.public_key());
	let bob_shared_secret = bob.shared_secret(&alice.public_key());
	assert_eq!(alice_shared_secret, bob_shared_secret);
}
