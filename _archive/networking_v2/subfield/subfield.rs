use crate::arr;
use crate::crypto::*;
use crate::vector::*;
use futures::task::{Context, Poll, Waker};
use futures::{Stream, StreamExt};
use getset::{Getters, Setters};
use libp2p::kad::store::MemoryStore;
use libp2p::{
	gossipsub,
	identity::Keypair,
	kad, ping, dcutr, autonat,
	request_response::{self, cbor::Behaviour},
	swarm::{NetworkBehaviour, SwarmEvent},
	StreamProtocol, Swarm,
};
#[cfg(not(target_arch = "wasm32"))]
use libp2p::{mdns, noise, tcp, yamux, relay};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;
use std::{io, time::Duration};

//#[cfg(not(target_arch = "wasm32"))]
//use tokio::{io, time::Duration};

//#[derive(Debug, Serialize, Deserialize)]
//pub struct SubfieldRequest {
//	pub ping: String,
//}

//#[derive(Debug, Serialize, Deserialize)]
//pub struct SubfieldResponse {
//	pub pong: String,
//}
#[derive(Debug)]
pub enum SubfieldError {
	NoSigner,
	NoCosigner,
	NoTangent,
}

/**
 * Subfield
*/
pub type Field = Option<U256>;

#[derive(Debug, Serialize, Deserialize, Getters, Setters)]
pub struct Subfield {
	#[getset(get = "pub")]
	signer: Field,
	#[getset(get = "pub")]
	cosigner: Field,
	#[getset(get = "pub")]
	tangent: Field,
}

impl Subfield {
	// Check if the entry has at least one of the three fields
	pub fn is_empty(&self) -> bool {
		self.signer().is_none()
			&& self.cosigner().is_none()
			&& self.tangent().is_none()
	}

	pub fn is_some(&self) -> bool {
		!self.is_empty()
	}

	// Check if the entry has a signer, cosigner, and tangent
	pub fn is_complete(&self) -> Result<(), SubfieldError> {
		if self.signer().is_none() {
			return Err(SubfieldError::NoSigner);
		}
		if self.cosigner().is_none() {
			return Err(SubfieldError::NoCosigner);
		}
		if self.tangent().is_none() {
			return Err(SubfieldError::NoTangent);
		}
		Ok(())
	}

	// hash the whole subfield
	pub fn hash(&self) -> U256 {
		let mut fields: Vec<&[u8]> = Vec::new();

		// signer, cosigner, and tangent are either unpacked U256 or [0;32]
		for method in [Self::signer, Self::cosigner, Self::tangent].iter() {
			if let Some(value) = method(self) {
				fields.push(value.data_u8().as_ref());
			} else {
				fields.push(&[0; 32]);
			}
		}

		hash::hash(arr::concat(&fields).as_ref())
	}

	// return a list of the hashes (or keys) of all possible combinations of the subfield
	pub fn hashes(&self) -> Vec<U256> {
		let mut result = Vec::new();

		let zero = &Some(U256::zero());
		let signer = self.signer();
		let cosigner = self.cosigner();
		let tangent = self.tangent();

		for (a, b, c) in &[
			// singles - hash(signer), hash(cosigner), hash(tangent)
			(signer, zero, zero),
			(zero, cosigner, zero),
			(zero, zero, tangent),
			// doubles - hash(all possible combinations of two fields)
			(signer, cosigner, zero),
			(signer, zero, tangent),
			(zero, cosigner, tangent),
			// triple - hash(signer, tangent, cosigner)
			(signer, cosigner, tangent),
		] {
			if let (Some(a), Some(b), Some(c)) =
				(a.as_ref(), b.as_ref(), c.as_ref())
			{
				result.push(hash::hash(
					arr::concat(&[a.data_u8(), b.data_u8(), c.data_u8()])
						.as_ref(),
				));
			}
		}

		result
	}
}

/**
 * SubfieldBehaviour
*/
#[derive(NetworkBehaviour)]
pub struct SubfieldBehaviour {
	//pub subfield: Behaviour<SubfieldRequest, SubfieldResponse>,
	pub ping: ping::Behaviour,
	pub kademlia: kad::Behaviour<MemoryStore>,
	pub gossipsub: gossipsub::Behaviour,
	pub dcutr: dcutr::Behaviour,
	pub autonat: autonat::Behaviour,
	#[cfg(not(target_arch = "wasm32"))]
	pub mdns: mdns::tokio::Behaviour,
	#[cfg(not(target_arch = "wasm32"))]
	pub relay: relay::Behaviour,
}

impl SubfieldBehaviour {
	pub fn new(key: &Keypair) -> Self {
		
		let local_peer_id = key.public().to_peer_id();
		
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
			dcutr: dcutr::Behaviour::new(local_peer_id.clone()),
			autonat: autonat::Behaviour::new(local_peer_id.clone(), autonat::Config::default()),
			#[cfg(not(target_arch = "wasm32"))]
			mdns: mdns::tokio::Behaviour::new(
				mdns::Config::default(),
				key.public().to_peer_id(),
			)
			.unwrap(),
			#[cfg(not(target_arch = "wasm32"))]
			relay: relay::Behaviour::new(local_peer_id.clone(), relay::Config::default()),
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
