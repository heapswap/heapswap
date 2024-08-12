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
	pub subfield:
		Behaviour<subfield::SubfieldRequest, subfield::SubfieldResponse>,

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

		let mut behaviour = SubfieldBehaviour {
			subfield: Behaviour::new(
				[(
					StreamProtocol::new("/subfield/1.0.0"),
					request_response::ProtocolSupport::Full,
				)],
				request_response::Config::default(),
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

		// Set the Kademlia mode
		// #[cfg(feature = "browser")]
		// {
		// 	behaviour.subfield.set_mode(Some(kad::Mode::Client));
		// }
		// #[cfg(feature = "server")]
		// {
		// 	behaviour.subfield.set_mode(Some(kad::Mode::Server));
		// }

		behaviour
	}
}
