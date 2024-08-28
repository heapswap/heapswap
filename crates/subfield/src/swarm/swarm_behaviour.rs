use super::super::*;
use crate::arr;
use crate::crypto::*;
use crate::*;
use futures::task::{Context, Poll, Waker};
use futures::{Stream, StreamExt};
use getset::{Getters, Setters};
use libp2p::kad::store::MemoryStore;
use libp2p::{
	autonat, dcutr, gossipsub,
	identity::Keypair,
	kad, noise, ping,
	request_response::{self, cbor::Behaviour},
	swarm::{NetworkBehaviour, SwarmEvent},
	yamux, StreamProtocol, Swarm,
};

#[cfg(feature = "server")]
use libp2p::{mdns, relay};

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;
use std::{io, time::Duration};

/**
 * SubfieldBehaviour
*/
#[derive(NetworkBehaviour)]
pub struct SubfieldBehaviour {
	// subfield
	pub subfield: request_response::cbor::Behaviour<
		proto::SubfieldRequest,
		proto::SubfieldResponse,
	>,

	// utils
	pub kad: kad::Behaviour<MemoryStore>,
	// pub pubsub: gossipsub::Behaviour,

	// networking
	pub ping: ping::Behaviour,
	pub dcutr: dcutr::Behaviour,
	pub autonat: autonat::Behaviour,
	#[cfg(feature = "server")]
	pub mdns: mdns::tokio::Behaviour,
	#[cfg(feature = "server")]
	pub relay: relay::Behaviour,
}

impl SubfieldBehaviour {
	pub fn new(key: &Keypair) -> Self {
		let local_peer_id = key.public().to_peer_id();

		/*
		// To content-address message, we can take the hash of message and use it as an ID.
		let _message_id_fn = |message: &gossipsub::Message| {
			let mut s = DefaultHasher::new();
			message.data.hash(&mut s);
			gossipsub::MessageId::from(s.finish().to_string())
		};

		// Set a custom gossipsub configuration
		let gossipsub_config = gossipsub::ConfigBuilder::default()
			.heartbeat_interval(Duration::from_secs(10))
			.validation_mode(gossipsub::ValidationMode::Strict)
			//.message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
			.build()
			.map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

		// build a gossipsub network behaviour
		let gossipsub = gossipsub::Behaviour::new(
			gossipsub::MessageAuthenticity::Signed(key.clone()),
			gossipsub_config,
		)?;
		*/

		let kad_config =
			kad::Config::new(StreamProtocol::new("/subfield/kad/1.0.0"));

		let mut behaviour = SubfieldBehaviour {
			subfield: Behaviour::new(
				[(
					StreamProtocol::new("/subfield/1.0.0"),
					request_response::ProtocolSupport::Full,
				)],
				request_response::Config::default(),
			),
			//pubsub: gossipsub,
			kad: kad::Behaviour::with_config(
				local_peer_id.clone(),
				MemoryStore::new(local_peer_id.clone()),
				kad_config,
			),
			ping: ping::Behaviour::new(ping::Config::new()),
			dcutr: dcutr::Behaviour::new(local_peer_id.clone()),
			autonat: autonat::Behaviour::new(
				local_peer_id.clone(),
				autonat::Config::default(),
			),
			#[cfg(feature = "server")]
			mdns: mdns::tokio::Behaviour::new(
				mdns::Config::default(),
				key.public().to_peer_id(),
			)
			.unwrap(),
			#[cfg(feature = "server")]
			relay: relay::Behaviour::new(
				local_peer_id.clone(),
				relay::Config::default(),
			),
		};

		// set mode
		#[cfg(feature = "client")]
		{
			behaviour.kad.set_mode(Some(kad::Mode::Client));
		}
		#[cfg(feature = "server")]
		{
			behaviour.kad.set_mode(Some(kad::Mode::Server));
		}

		behaviour
	}
}
