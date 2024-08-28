use super::SubfieldConfig;
use crate::*;

use futures::{prelude::*, StreamExt};
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;

use super::swarm_behaviour::*;
use bytes::Bytes;

use libp2p::{
	core::{muxing::StreamMuxerBox, transport, upgrade, Transport as _},
	gossipsub,
	kad::QueryResult as KadQueryResult,
	noise, ping,
	swarm::{NetworkBehaviour, SwarmEvent},
	yamux, Multiaddr, Swarm, SwarmBuilder, Transport as _, TransportExt,
};
// use libp2p_core::Transport as _;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "server")]
use {
	libp2p::{
		mdns,
		tcp,
		// dns::tokio::Transport as DnsTransport
	},
	// libp2p::websocket,
	libp2p_webrtc as webrtc,
	tracing_subscriber::EnvFilter,
};
#[cfg(feature = "client")]
use {
	// libp2p::websocket_websys,
	libp2p_webrtc_websys as webrtc_websys,
};

pub type SubfieldSwarm = Swarm<SubfieldBehaviour>;
pub type SubfieldSwarmEvent = SwarmEvent<SubfieldBehaviourEvent>;
pub type ThreadsafeSubfieldSwarm = Arc<Mutex<SubfieldSwarm>>;
pub type ThreadsafeSubfieldSwarmLock<'a> = MutexGuard<'a, SubfieldSwarm>;

#[derive(Debug)]
pub enum SubfieldCreateSwarmError {
	FailedToLockSwarm,
	FailedToDialBootstrapAddrs,
}

#[derive(Debug, Clone)]
pub enum SubfieldSwarmMode {
	Client,
	Server,
}

const IDLE_CONNECTION_TIMEOUT: u64 = 60;

/**
 * Create a Subfield Swarm
 * Switches between client and server based on the feature flags
*/
// #[wasm_bindgen]
pub async fn create_swarm(
	swarm_config: SubfieldConfig,
) -> eyre::Result<SubfieldSwarm> {
	#![allow(unused_assignments)]
	let mut swarm: eyre::Result<SubfieldSwarm> =
		Err(eyr!("Failed to create swarm"));
	#[cfg(feature = "client")]
	{
		swarm = create_client(swarm_config.clone()).await
	}
	#[cfg(feature = "server")]
	{
		swarm = create_server(swarm_config.clone()).await
	}

	if swarm.is_err() {
		return swarm;
	} else {
		tracing::info!("Successfully created swarm");
		return swarm;
	}
}

/**
 * Client - Wasm
*/
#[cfg(feature = "client")]
async fn create_client(
	swarm_config: SubfieldConfig,
) -> eyre::Result<SubfieldSwarm> {
	use libp2p::core::upgrade;
	use std::time::Duration;
	use tracing::instrument::WithSubscriber;

	#[cfg(target_arch = "wasm32")]
	{
		let keypair = swarm_config
			.keypair
			.to_libp2p_keypair()
			.map_err(|e| eyr!(e.to_string()))?;

		let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
			.with_wasm_bindgen()
			.with_other_transport(|key| {
				// webtransport
				// let config = webtransport_websys::Config::new(&key);
				// let transport = webtransport_websys::Transport::new(config).boxed();
				// Ok(transport)

				// websockets
				let transport = websocket_websys::Transport::default()
					.upgrade(upgrade::Version::V1)
					.authenticate(noise::Config::new(&key)?)
					.multiplex(yamux::Config::default())
					.boxed();

				// webrtc
				// webrtc_websys::Transport::new(webrtc_websys::Config::new(&key)).boxed()
			})?
			.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))?
			.with_swarm_config(|c| {
				c.with_idle_connection_timeout(Duration::from_secs(
					IDLE_CONNECTION_TIMEOUT,
				))
			})
			.build();

		tracing::info!("Successfully created swarm");

		return Ok(swarm);
	}
	#[cfg(not(target_arch = "wasm32"))]
	panic!("client not implemented on non-wasm32")
}

/**
 * Server - Non-Wasm
*/
#[cfg(feature = "server")]
async fn create_server(
	swarm_config: SubfieldConfig,
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
			// websocket
			.with_websocket(
				(libp2p::tls::Config::new, libp2p::noise::Config::new),
				libp2p::yamux::Config::default,
			)
			.await?
			// webrtc
			/*
			.with_other_transport(|id_keys| {
				Ok(webrtc::tokio::Transport::new(
					id_keys.clone(),
					webrtc::tokio::Certificate::generate(&mut thread_rng())?,
				)
				.map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn))))
			})?*/
			.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))
			.map_err(|e| eyr!(e.to_string()))?
			.with_swarm_config(|c| {
				c.with_idle_connection_timeout(Duration::from_secs(
					IDLE_CONNECTION_TIMEOUT * 3,
				))
			})
			.build();

	for addr in swarm_config.listen_addresses {
		swarm.listen_on(addr.parse::<Multiaddr>()?)?;
	}
	Ok(swarm)
}
