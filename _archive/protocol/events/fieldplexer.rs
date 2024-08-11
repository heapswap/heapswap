use super::entry::*;
use super::entry::*;
use super::subfield::SubfieldBehaviour;
use super::subfield::*;
use super::swarm_create::*;
use crate::arr;
use crate::crypto::*;
use crate::crypto::*;
use crate::vector::*;
use bytes::Bytes;
use dashmap::DashMap;
use futures::task::{Context, Poll, Waker};
use futures::{Stream, StreamExt};
use getset::{Getters, MutGetters, Setters};
use gloo::events::*;
use hash::hash;
use js_sys::Function;
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
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum FieldplexError<T> {
	InvalidCallback,
	SubfieldCreateSwarmError(SubfieldCreateSwarmError),
	SubfieldError(SubfieldError),
	SubfieldEntryError(SubfieldEntryError),
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

pub type Callback = dyn FnMut(&web_sys::Event) + 'static;

/**
 * The purpose of the Fieldplexer is to split Subfield entries into Field entries and vice versa.
*/
#[derive(Getters, Setters, MutGetters)]
pub struct Fieldplexer {
	// swarm
	#[getset(get = "pub", get_mut = "pub")]
	swarm: SubfieldSwarm,
	// channels
	#[getset(get = "pub")]
	incoming: Fieldplex<FieldEntry>,
	#[getset(get = "pub")]
	outgoing: Fieldplex<SubfieldEntry>,
	// subscriptions
	current_subscriptions: DashMap<String, String>,
	// dom element
	#[getset(get = "pub")]
	dom_element: web_sys::Element,
}

impl Fieldplexer {
	pub fn new(swarm: SubfieldSwarm) -> Self {
		let dom_element = web_sys::window()
			.unwrap()
			.document()
			.unwrap()
			.create_element("div")
			.unwrap();
		Fieldplexer {
			swarm,
			incoming: Fieldplex::new(),
			outgoing: Fieldplex::new(),
			current_subscriptions: DashMap::new(),
			dom_element,
		}
	}


	pub fn publish(
		&self,
		entry: SubfieldEntry,
	) -> Result<(), FieldplexError<SubfieldEntry>> {
		let mut swarm = get_mut_swarm(self.swarm())
			.map_err(FieldplexError::SubfieldCreateSwarmError)?;

		// Check if the subfield is complete
		match entry.subfield().is_complete() {
			Ok(_) => {}
			Err(e) => {
				return Err(FieldplexError::SubfieldError(e));
			}
		}

		// Check if the signer signature is valid
		match entry.signer_signature_is_valid() {
			Ok(_) => {}
			Err(e) => {
				return Err(FieldplexError::SubfieldEntryError(e));
			}
		}

		// publish to all the topic hashes
		let subfield_hash = entry.subfield().hash().to_string();
		let hashes = entry.subfield().hashes();
		let data = bincode::serialize(entry.entry().clone()).unwrap();
		for hash in hashes.iter() {
			let topic = gossipsub::IdentTopic::new(hash.to_string());

			// publish to the topic
			swarm.behaviour_mut().gossipsub.publish(topic, data.clone());
		}

		Ok(())
	}

	pub fn subscribe(
		&mut self,
		subfield: Subfield,
		callback: JsValue,
	) -> Result<EventListener, FieldplexError<SubfieldEntry>> {
		let mut swarm = get_mut_swarm(self.swarm())
			.map_err(FieldplexError::SubfieldCreateSwarmError)?;

		// subscribe to the topic hash
		let subfield_hash = subfield.hash().to_string();
		let topic = gossipsub::IdentTopic::new(subfield_hash.clone());
		swarm.behaviour_mut().gossipsub.subscribe(&topic);

		// Ensure _callback is a function
		let callback_fn = callback
			.dyn_into::<Function>()
			.map_err(|_| FieldplexError::InvalidCallback)?;

		// create the event listener
		let event_listener = EventListener::new(
			self.dom_element(),
			subfield_hash,
			move |event| {
				callback_fn.call1(&JsValue::NULL, &event).unwrap();
			},
		);

		Ok(event_listener)
	}

	//pub fn recieve() -> Result<(), _> {

	//}
}
