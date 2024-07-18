use crate::crypto::*;
use crate::u256::*;
use bytes::Bytes;
use futures::task::{Context, Poll, Waker};
use futures::{Stream, StreamExt};
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
	public_data: PublicEntryData,
	private_data: PrivateEntryData,
}

// Public entry data is always unencrypted
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicEntryData {
	#[serde(with = "serde_bytes")]
	signer_signature: keys::SignatureArr,
	#[serde(with = "serde_bytes")]
	cosigner_signature: Option<keys::SignatureArr>,
	private_is_private: bool,
	created_ts: chrono::DateTime<chrono::Utc>,
	updated_ts: chrono::DateTime<chrono::Utc>,
}

// Private entry data is optionally encrypted, depending on the private_is_private flag
#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateEntryData {
	seed_plaintext: Option<Bytes>,
	datatype: Option<u32>,
	data: Option<Bytes>,
}
