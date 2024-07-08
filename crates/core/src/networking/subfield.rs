use libp2p::{
	mdns, request_response,
	request_response::cbor::Behaviour,
	swarm::{NetworkBehaviour, SwarmEvent},
	Swarm,
};
use serde::{Deserialize, Serialize};

/*
#[derive(Debug, Serialize, Deserialize)]
pub struct Field{
	pub signer: String,
	pub cosigner: String,
	pub tangent: String,
}
*/

#[derive(Debug, Serialize, Deserialize)]
pub struct SubfieldRequest {
	pub ping: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubfieldResponse {
	pub pong: String,
}

#[derive(NetworkBehaviour)]
pub struct SubfieldBehaviour {
	pub subfield: Behaviour<SubfieldRequest, SubfieldResponse>,
	#[cfg(not(target_arch = "wasm32"))]
	pub mdns: mdns::tokio::Behaviour,
}

pub fn swarm_handle_subfield_event(
	swarm: &mut Swarm<SubfieldBehaviour>,
	event: SwarmEvent<SubfieldBehaviourEvent>,
) {
	match event {
		/*
			 Subfield
		*/
		SwarmEvent::Behaviour(SubfieldBehaviourEvent::Subfield(event)) => {
			match event {
				request_response::Event::Message { peer, message } => {
					println!("Received message from {:?}: {:?}", peer, message);
				}
				_ => {}
			}
		}
		/*
			 System
		*/
		SwarmEvent::NewListenAddr {
			listener_id,
			address,
		} => {
			println!("{:?} listening on {:?}", swarm.local_peer_id(), address);
		}
		SwarmEvent::ConnectionEstablished {
			peer_id,
			connection_id,
			endpoint,
			num_established,
			concurrent_dial_errors,
			established_in,
		} => {
			println!(
				"Connection established with {:?} ({:?})",
				peer_id, num_established
			);
		}
		/*
			mDNS
		*/
		#[cfg(not(target_arch = "wasm32"))]
		SwarmEvent::ConnectionClosed {
			peer_id,
			cause,
			num_established,
			endpoint,
			..
		} => {
			println!(
				"Connection closed with {:?} ({:?})",
				peer_id, num_established
			);
		}
		SwarmEvent::Behaviour(SubfieldBehaviourEvent::Mdns(
			mdns::Event::Discovered(list),
		)) => {
			for (peer_id, multiaddr) in list {
				println!("mDNS discovered a new peer: {peer_id}");
				let _ = swarm.dial(multiaddr);
			}
		}
		#[cfg(not(target_arch = "wasm32"))]
		SwarmEvent::Behaviour(SubfieldBehaviourEvent::Mdns(
			mdns::Event::Expired(list),
		)) => {
			for (peer_id, multiaddr) in list {
				println!("mDNS discover peer has expired: {peer_id}");
			}
		}
		_ => {}
	}
}
