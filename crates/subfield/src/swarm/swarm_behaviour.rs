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
	// request_response::{self, cbor::Behaviour},
	swarm::behaviour::toggle::Toggle,
	swarm::{NetworkBehaviour, SwarmEvent},
	yamux, StreamProtocol, Swarm,
};

#[cfg(feature = "server")]
use libp2p::{mdns, relay};

use libp2p_stream as stream;

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;
use std::{io, time::Duration};

pub const SUBFIELD_PROTOCOL: StreamProtocol =
	StreamProtocol::new("/subfield/1.0.0");

/*
   SubfieldNetworkBehaviour
*/
#[derive(NetworkBehaviour)]
pub struct SubfieldNetworkBehaviour {
	
	// subfield
	pub subfield : stream::Behaviour,
	
	// dht
	pub kad: kad::Behaviour<MemoryStore>,

	// networking
	pub ping: ping::Behaviour,
	pub dcutr: dcutr::Behaviour,
	pub autonat: autonat::Behaviour,
	#[cfg(feature = "server")]
	pub mdns: mdns::tokio::Behaviour,
	#[cfg(feature = "server")]
	pub relay: relay::Behaviour,
}

impl SubfieldNetworkBehaviour {
	pub fn new(key: &Keypair) -> Self {
		let local_peer_id = key.public().to_peer_id();

		let kad_config =
			kad::Config::new(StreamProtocol::new("/subfield/kad/1.0.0"));

		let mut behaviour = SubfieldNetworkBehaviour {
			subfield: stream::Behaviour::new(),
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
			// behaviour.chord.set_mode(SubfieldMode::Client);
		}
		#[cfg(feature = "server")]
		{
			behaviour.kad.set_mode(Some(kad::Mode::Server));
			// behaviour.chord.set_mode(SubfieldMode::Server);
		}

		behaviour
	}
}
