use super::entry::*;
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
 * Field
*/
pub type Field = Option<U256>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Subfield {
	signer: Field,
	cosigner: Field,
	tangent: Field,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubfieldEntry {
	subfield: Subfield,
	entry: Entry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldEntry {
	field: Field,
	entry: Vec<u8>,
}

#[derive(Debug)]
pub enum FieldplexError<T> {
	NoSigner,
	NoCosigner,
	NoTangent,
	FailedToSend(flume::SendError<T>),
}

pub struct Fieldplex<T> {
	tx: flume::Sender<T>,
	rx: flume::Receiver<T>,
}

impl<T> Fieldplex<T> {
	pub fn new() -> Self {
		let (tx, rx) = flume::unbounded();
		Fieldplex { tx, rx }
	}

	pub fn rx(&self) -> &flume::Receiver<T> {
		&self.rx
	}

	pub fn tx(&self) -> &flume::Sender<T> {
		&self.tx
	}

	pub fn send(&self, entry: T) -> Result<(), FieldplexError<T>> {
		self.tx.send(entry).map_err(FieldplexError::FailedToSend)
	}
}

/**
 * The purpose of the Fieldplexer is to split Subfield entries into Field entries and vice versa.
*/
pub struct Fieldplexer {
	incoming: Fieldplex<FieldEntry>,
	outgoing: Fieldplex<SubfieldEntry>,
}

impl Fieldplexer {
	pub fn new() -> Self {
		Fieldplexer {
			incoming: Fieldplex::new(),
			outgoing: Fieldplex::new(),
		}
	}

	pub fn tx(&self) -> &flume::Sender<SubfieldEntry> {
		self.outgoing.tx()
	}

	pub fn rx(&self) -> &flume::Receiver<FieldEntry> {
		self.incoming.rx()
	}

	pub fn send(
		&self,
		entry: SubfieldEntry,
	) -> Result<(), FieldplexError<SubfieldEntry>> {
		if entry.subfield.signer.is_none() {
			return Err(FieldplexError::NoSigner);
		}

		self.outgoing.send(entry)
	}
}
