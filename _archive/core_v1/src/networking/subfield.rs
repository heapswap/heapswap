use futures::task::{Context, Poll, Waker};
use futures::{Stream, StreamExt};
use libp2p::{
	identity::Keypair,
	request_response::{self, cbor::Behaviour},
	swarm::{NetworkBehaviour, SwarmEvent},
	StreamProtocol, Swarm,
};
#[cfg(not(target_arch = "wasm32"))]
use libp2p::{mdns, noise, tcp, yamux};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::yield_now;
use tokio_with_wasm::alias as tokio;

pub type ThreadsafeSubfieldSwarm = Arc<Mutex<Swarm<SubfieldBehaviour>>>;

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

impl SubfieldBehaviour {
	pub fn new(key: &Keypair) -> Self {
		let subfield = Behaviour::new(
			[(
				StreamProtocol::new("/subfield/1.0.0"),
				request_response::ProtocolSupport::Full,
			)],
			request_response::Config::default(),
		);

		SubfieldBehaviour {
			subfield,
			#[cfg(not(target_arch = "wasm32"))]
			mdns: mdns::tokio::Behaviour::new(
				mdns::Config::default(),
				key.public().to_peer_id(),
			)
			.unwrap(),
		}
	}
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
		/*
			mDNS
		*/
		#[cfg(not(target_arch = "wasm32"))]
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

// due to the swarm needing to be wrapped in a mutex for use with axum,
// we need to poll the swarm instead of using the swarm.next() method
async fn poll_swarm(
	swarm: &mut Swarm<SubfieldBehaviour>,
) -> Option<SwarmEvent<SubfieldBehaviourEvent>> {
	match swarm
		.poll_next_unpin(&mut Context::from_waker(&futures::task::noop_waker()))
	{
		Poll::Ready(Some(event)) => Some(event),
		Poll::Ready(None) => None,
		Poll::Pending => None,
	}
}

// spawn a tokio task that will poll the swarm and handle events
pub fn spawn_swarm_loop(
	swarm: ThreadsafeSubfieldSwarm,
) -> tokio::task::JoinHandle<()> {
	tokio::spawn(async move {
		loop {
			let event = {
				let mut lock = swarm.lock().await;
				poll_swarm(&mut lock).await
			};

			if let Some(event) = event {
				let mut lock = swarm.lock().await;
				swarm_handle_subfield_event(&mut *lock, event);
			}

			let _ = yield_now().await;
		}
	})
}
