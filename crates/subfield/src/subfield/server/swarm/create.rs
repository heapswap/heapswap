use crate::*;

use futures::{prelude::*, StreamExt};
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;

use super::behaviour::*;
use bytes::Bytes;

use libp2p::{
	core::Transport,
	core::{muxing::StreamMuxerBox, transport, upgrade},
	gossipsub,
	kad::QueryResult as KadQueryResult,
	noise, ping,
	swarm::{NetworkBehaviour, SwarmEvent},
	yamux, Multiaddr, Swarm, SwarmBuilder, Transport as _, TransportExt,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "browser")]
use {
	libp2p::websocket_websys,
	// libp2p_webrtc_websys as webrtc_websys,
};
#[cfg(feature = "server")]
use {
	libp2p::{mdns, tcp, websocket},
	// libp2p_webrtc as webrtc,
	tracing_subscriber::EnvFilter,
};

pub type SubfieldSwarm = Swarm<SubfieldBehaviour>;
pub type SubfieldSwarmEvent = SwarmEvent<SubfieldBehaviourEvent>;
#[cfg(feature = "server")]
pub type ThreadsafeSubfieldSwarm = Arc<Mutex<SubfieldSwarm>>;
#[cfg(feature = "server")]
pub type ThreadsafeSubfieldSwarmLock<'a> = MutexGuard<'a, SubfieldSwarm>;

#[derive(Debug)]
pub enum SubfieldCreateSwarmError {
	FailedToLockSwarm,
	FailedToDialBootstrapAddrs,
}

#[derive(Clone)]
pub enum SubfieldSwarmMode {
	Client,
	Server,
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct SubfieldSwarmConfig {
	pub mode: SubfieldSwarmMode,
	pub keypair: crypto::Keypair,
	pub listen_addresses: Vec<String>,
	pub bootstrap_multiaddrs: Vec<String>,
}

const IDLE_CONNECTION_TIMEOUT: u64 = 600;

/**
 * Create a Subfield Swarm
 * Switches between client and server based on the feature flags
*/
#[wasm_bindgen]
pub async fn create_swarm(
	swarm_config: SubfieldSwarmConfig,
) -> eyre::Result<SubfieldSwarm> {
	#![allow(unused_assignments)]
	let mut swarm: Result<Swarm<SubfieldBehaviour>, EReport> =
		Err(eyr!("Failed to create swarm"));
	#[cfg(feature = "browser")]
	{
		swarm = create_client(swarm_config).await;
	}
	#[cfg(feature = "server")]
	{
		swarm = create_server(swarm_config).await;
	}
	swarm
}

/**
 * Client - Wasm
*/
#[cfg(feature = "browser")]
async fn create_client(
	swarm_config: SubfieldSwarmConfig,
) -> eyre::Result<SubfieldSwarm> {
	use libp2p::core::upgrade;
	use std::time::Duration;
	use tracing::instrument::WithSubscriber;

	#[cfg(target_arch = "wasm32")]
	{
		let keypair = swarm_config.keypair.to_libp2p_keypair();

		let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
			.with_wasm_bindgen()
			.with_other_transport(|key| {
				// let config = webtransport_websys::Config::new(&key);
				// let transport = webtransport_websys::Transport::new(config).boxed();
				// Ok(transport)
				let transport = websocket_websys::Transport::default()
					.upgrade(upgrade::Version::V1)
					.authenticate(noise::Config::new(&key)?)
					.multiplex(yamux::Config::default())
					.boxed();
				Ok(transport)
			})?
			.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))?
			.with_swarm_config(|c| {
				c.with_idle_connection_timeout(Duration::from_secs(
					IDLE_CONNECTION_TIMEOUT,
				))
			})
			.build();

		tracing::info!("Successfully created swarm");

		let mut success = false;

		for addr in swarm_config.bootstrap_multiaddrs {
			tracing::info!("Dialing bootstrap URL: {:?}", addr);
			match subfield.dial(
				addr.parse::<Multiaddr>()
					.map_err(|_| eyr!("Failed to parse bootstrap multiaddr"))?,
			) {
				Ok(_) => {
					tracing::info!(
						"Successfully dialed bootstrap URL: {:?}",
						addr
					);
					success = true;
					break;
				}
				Err(_) => {}
			}
		}
		if success {
			return Ok(subfield);
		} else {
			return Err(eyr!("No bootstrap nodes dialed"));
		}
	}
	#[cfg(not(target_arch = "wasm32"))]
	panic!("client on non-wasm32")
}

/**
 * Server - Non-Wasm
*/
#[cfg(feature = "server")]
async fn create_server(
	swarm_config: SubfieldSwarmConfig,
) -> eyre::Result<SubfieldSwarm> {
	let keypair = swarm_config.keypair.to_libp2p_keypair().unwrap();

	let mut swarm =
		libp2p::SwarmBuilder::with_existing_identity(keypair.clone())
			.with_tokio()
			.with_tcp(
				tcp::Config::default(),
				noise::Config::new,
				yamux::Config::default,
			)?
			.with_websocket(
				(libp2p::tls::Config::new, libp2p::noise::Config::new),
				libp2p::yamux::Config::default,
			)
			.await?
			// .with_other_transport(|id_keys| {
			// 	Ok(webrtc::tokio::Transport::new(
			// 		id_keys.clone(),
			// 		webrtc::tokio::Certificate::generate(
			// 			&mut rand::thread_rng(),
			// 		)?,
			// 	)
			// 	.map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn))))
			// })?
			.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))
			.map_err(|e| eyr!(e.to_string()))?
			.with_swarm_config(|c| {
				c.with_idle_connection_timeout(Duration::from_secs(
					IDLE_CONNECTION_TIMEOUT,
				))
			})
			.build();

	for addr in swarm_config.listen_addresses {
		swarm.listen_on(addr.parse::<Multiaddr>()?)?;
	}
	Ok(swarm)
}
