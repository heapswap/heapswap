use super::super::*;
use crate::arr;
use crate::crypto::*;
use crate::vector::*;
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
	pub subfield: Behaviour<SubfieldRequest, SubfieldResponse>,
	pub ping: ping::Behaviour,
	pub kademlia: kad::Behaviour<MemoryStore>,
	pub gossipsub: gossipsub::Behaviour,
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

		// Content-address messages (No two messages of the same content will be propagated)
		//let _message_id_fn = |message: &gossipsub::Message| {
		//	crate::crypto::hash::hash(message.data.as_ref()).to_string()
		//};

		// Gossipsub
		let gossipsub_config = gossipsub::ConfigBuilder::default()
			.heartbeat_interval(Duration::from_secs(10))
			.validation_mode(gossipsub::ValidationMode::Strict)
			.protocol_id("/subfield/pubsub/v1.1.0", gossipsub::Version::V1_1)
			//.message_id_fn(message_id_fn)
			.build()
			.map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))
			.unwrap();
		let gossipsub = gossipsub::Behaviour::new(
			gossipsub::MessageAuthenticity::Signed(key.clone()),
			gossipsub_config,
		)
		.unwrap();

		// Kademlia
		let mut kad_config = kad::Config::default();
		kad_config.set_protocol_names(vec![StreamProtocol::new(
			"/subfield/kad/1.0.0",
		)]);
		let kademlia = kad::Behaviour::with_config(
			key.public().to_peer_id(),
			MemoryStore::new(key.public().to_peer_id()),
			kad_config,
		);

		let mut behaviour = SubfieldBehaviour {
			subfield: Behaviour::new(
				[(
					StreamProtocol::new("/subfield/1.0.0"),
					request_response::ProtocolSupport::Full,
				)],
				request_response::Config::default(),
			),
			ping: ping::Behaviour::new(ping::Config::new()),
			gossipsub,
			kademlia: kad::Behaviour::new(
				key.public().to_peer_id(),
				MemoryStore::new(key.public().to_peer_id()),
			),
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

		// Set the Kademlia mode
		#[cfg(feature = "browser")]
		{
			behaviour.kademlia.set_mode(Some(kad::Mode::Client));
		}
		#[cfg(feature = "server")]
		{
			behaviour.kademlia.set_mode(Some(kad::Mode::Server));
		}

		behaviour
	}
}
