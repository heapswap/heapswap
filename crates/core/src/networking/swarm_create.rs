#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]
use futures::{prelude::*, stream::StreamExt};
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;

use super::subfield::*;
use crate::{arr, crypto::keys::Keypair};
use bytes::Bytes;
use libp2p::{
	gossipsub,
	kad::QueryResult as KadQueryResult,
	noise, ping,
	swarm::{NetworkBehaviour, SwarmEvent},
	yamux, Multiaddr, Swarm, Transport,
};
#[cfg(not(target_arch = "wasm32"))]
use libp2p::{mdns, tcp};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

pub fn heapswap_keypair_to_libp2p_keypair(
	keypair: &Keypair,
) -> libp2p::identity::Keypair {
	libp2p::identity::Keypair::ed25519_from_bytes(
		keypair.private_key().u256().unpacked().clone(),
	)
	.unwrap()
}

#[derive(Clone)]
pub struct SwarmConfig {
	pub keypair: Keypair,
	#[cfg(not(target_arch = "wasm32"))]
	pub listen_addresses: Vec<String>,
	#[cfg(target_arch = "wasm32")]
	pub bootstrap_multiaddrs: Vec<String>,
}

/**
 * Wasm
*/
#[cfg(target_arch = "wasm32")]
pub async fn swarm_create(
	swarm_config: SwarmConfig,
) -> Result<Swarm<SubfieldBehaviour>, Box<dyn Error>> {
	use libp2p::core::upgrade;
	use std::time::Duration;

	let mut swarm = libp2p::SwarmBuilder::with_existing_identity(
		heapswap_keypair_to_libp2p_keypair(&swarm_config.keypair),
	)
	.with_wasm_bindgen()
	.with_other_transport(|key| {
		//Ok(libp2p::webtransport_websys::Transport::new(libp2p::webtransport_websys::Config::new(&key)))
		Ok(libp2p::websocket_websys::Transport::default()
			.upgrade(upgrade::Version::V1)
			.authenticate(
				noise::Config::new(&key)
					.map_err(|_| "Failed to authenticate with keypair")?,
			)
			.multiplex(yamux::Config::default())
			.boxed())
	})?
	.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))?
	.with_swarm_config(|c| {
		c.with_idle_connection_timeout(Duration::from_secs(60))
	})
	.build();

	tracing::debug!("Successfully created swarm");

	for addr in swarm_config.bootstrap_multiaddrs {
		match swarm.dial(
			addr.parse::<Multiaddr>()
				.map_err(|_| "Failed to parse bootstrap multiaddr")?,
		) {
			Ok(_) => {
				tracing::debug!(
					"Successfully dialed bootstrap URL: {:?}",
					addr
				);
			}
			//Err(e) => eprintln!("Failed to dial bootstrap URL: {:?}", e),
			Err(_) => {}
		}
	}

	Ok(swarm)
}

/**
 * Non-Wasm
*/
#[cfg(not(target_arch = "wasm32"))]
pub async fn swarm_create(
	swarm_config: SwarmConfig,
) -> Result<Swarm<SubfieldBehaviour>, Box<dyn Error>> {
	use tokio::time::Duration;

	let mut swarm = libp2p::SwarmBuilder::with_existing_identity(
		heapswap_keypair_to_libp2p_keypair(&swarm_config.keypair),
	)
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
	.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))
	.map_err(|e| e.to_string())?
	.with_swarm_config(|c| {
		c.with_idle_connection_timeout(Duration::from_secs(60))
	})
	.build();

	for addr in swarm_config.listen_addresses {
		swarm.listen_on(addr.parse::<Multiaddr>()?)?;
	}

	Ok(swarm)
}
