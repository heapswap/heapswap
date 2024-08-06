use super::subfield::*;
use crate::crypto::*;
use crate::vector::*;
use bytes::Bytes;
use futures::task::{Context, Poll, Waker};
use futures::{Stream, StreamExt};
use getset::{Getters, Setters};
use libp2p::kad::store::MemoryStore;
use libp2p::{
	gossipsub,
	identity::Keypair,
	kad, ping,
	request_response::{self, cbor::Behaviour},
	swarm::{NetworkBehaviour, SwarmEvent},
	StreamProtocol, Swarm,
};
#[cfg(not(target_arch = "wasm32"))]
use libp2p::{mdns, noise, tcp, yamux};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;

/**
 * An entry object represents an entry in the DHT
*/
#[derive(Debug, Serialize, Deserialize, Getters, Setters)]
pub struct Entry {
	#[getset(get = "pub")]
	public_data: PublicEntryData,

	#[getset(get = "pub")]
	private_data: PrivateEntryData,
}

// Public entry data is always unencrypted
#[derive(Debug, Serialize, Deserialize, Getters, Setters)]
pub struct PublicEntryData {
	#[getset(get = "pub")]
	#[serde(with = "serde_bytes")]
	signer_signature: keys::SignatureArr,

	#[getset(get = "pub")]
	#[serde(with = "serde_bytes")]
	cosigner_signature: Option<keys::SignatureArr>,

	#[getset(get = "pub")]
	private_is_private: bool,

	#[getset(get = "pub")]
	created_ts: chrono::DateTime<chrono::Utc>,

	#[getset(get = "pub")]
	updated_ts: chrono::DateTime<chrono::Utc>,
}

// Private entry data is optionally encrypted, depending on the private_is_private flag
#[derive(Debug, Serialize, Deserialize, Getters, Setters)]
pub struct PrivateEntryData {
	#[getset(get = "pub")]
	seed_plaintext: Option<Bytes>,
	#[getset(get = "pub")]
	datatype: Option<u32>,
	#[getset(get = "pub")]
	data: Option<Bytes>,
}

#[derive(Debug)]
pub enum SubfieldEntryError {
	InvalidSignerSignature,
	FailedToParseSignerSignature,
	NoSignerSignature,
	InvalidCosignerSignature,
	FailedToParseCosignerSignature,
}

// A field entry is a field and an entry
// This is sent as gossipsub messages
#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct FieldEntry {
	#[getset(get = "pub")]
	field: Field,
	#[getset(get = "pub")]
	entry: Vec<u8>,
}

// A subfield entry is a subfield and an entry
#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct SubfieldEntry {
	#[getset(get = "pub")]
	subfield: Subfield,
	#[getset(get = "pub")]
	entry: Entry,
}

impl SubfieldEntry {
	// Check the entry signer signature
	pub fn signer_signature_is_valid(
		&self,
	) -> Result<bool, SubfieldEntryError> {
		let signer = keys::PublicKey::from_u256(
			self.subfield().signer().as_ref().unwrap().clone(),
		);
		let verified = signer.verify(
			self.subfield().hash().data_u8(),
			self.entry().public_data().signer_signature(),
		);
		match verified {
			Ok(res) => {
				if !res {
					return Err(SubfieldEntryError::InvalidSignerSignature);
				}
			}
			Err(_) => {
				return Err(SubfieldEntryError::FailedToParseSignerSignature);
			}
		}
		Ok(true)
	}

	// Check the entry cosigner signature
	pub fn cosigner_signature_is_valid(
		&self,
	) -> Result<bool, SubfieldEntryError> {
		if self.entry().public_data().cosigner_signature().is_none() {
			return Ok(true);
		}
		let cosigner = keys::PublicKey::from_u256(
			self.subfield().cosigner().as_ref().unwrap().clone(),
		);
		let verified = cosigner.verify(
			self.subfield().hash().data_u8(),
			self.entry()
				.public_data()
				.cosigner_signature()
				.as_ref()
				.unwrap(),
		);
		match verified {
			Ok(res) => {
				if !res {
					return Err(SubfieldEntryError::InvalidCosignerSignature);
				}
			}
			Err(_) => {
				return Err(SubfieldEntryError::FailedToParseCosignerSignature);
			}
		}
		Ok(true)
	}
}
