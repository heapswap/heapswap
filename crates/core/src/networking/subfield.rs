use crate::u256::*;
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

#[cfg(target_arch = "wasm32")]
use std::{io, time::Duration};

#[cfg(not(target_arch = "wasm32"))]
use tokio::{io, time::Duration};

//#[derive(Debug, Serialize, Deserialize)]
//pub struct SubfieldRequest {
//	pub ping: String,
//}

//#[derive(Debug, Serialize, Deserialize)]
//pub struct SubfieldResponse {
//	pub pong: String,
//}

#[derive(NetworkBehaviour)]
pub struct SubfieldBehaviour {
	//pub subfield: Behaviour<SubfieldRequest, SubfieldResponse>,
	pub ping: ping::Behaviour,
	pub kademlia: kad::Behaviour<MemoryStore>,
	pub gossipsub: gossipsub::Behaviour,
	#[cfg(not(target_arch = "wasm32"))]
	pub mdns: mdns::tokio::Behaviour,
}

impl SubfieldBehaviour {
	pub fn new(key: &Keypair) -> Self {
		// custom subfield protocol
		//let subfield = Behaviour::new(
		//	[(
		//		StreamProtocol::new("/subfield/1.0.0"),
		//		request_response::ProtocolSupport::Full,
		//	)],
		//	request_response::Config::default(),
		//);

		// Content-address messages (No two messages of the same content will be propagated)
		//let _message_id_fn = |message: &gossipsub::Message| {
		//	crate::crypto::hash::hash(message.data.as_ref()).to_string()
		//};

		// Gossipsub
		let gossipsub_config = gossipsub::ConfigBuilder::default()
			.heartbeat_interval(Duration::from_secs(10))
			.validation_mode(gossipsub::ValidationMode::Strict)
			//.message_id_fn(message_id_fn)
			.build()
			.map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))
			.unwrap();
		let gossipsub = gossipsub::Behaviour::new(
			gossipsub::MessageAuthenticity::Signed(key.clone()),
			gossipsub_config,
		)
		.unwrap();

		let mut behaviour = SubfieldBehaviour {
			ping: ping::Behaviour::new(ping::Config::new()),
			gossipsub,
			kademlia: kad::Behaviour::new(
				key.public().to_peer_id(),
				MemoryStore::new(key.public().to_peer_id()),
			),
			#[cfg(not(target_arch = "wasm32"))]
			mdns: mdns::tokio::Behaviour::new(
				mdns::Config::default(),
				key.public().to_peer_id(),
			)
			.unwrap(),
		};

		// Set the Kademlia mode
		#[cfg(target_arch = "wasm32")]
		{
			behaviour.kademlia.set_mode(Some(kad::Mode::Client));
		}
		#[cfg(not(target_arch = "wasm32"))]
		{
			behaviour.kademlia.set_mode(Some(kad::Mode::Server));
		}

		behaviour
	}
}
