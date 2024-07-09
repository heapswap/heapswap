use futures::TryFutureExt;
use libp2p::{
	core::{muxing::StreamMuxerBox, transport, upgrade},
	identity::Keypair,
	noise,
	request_response::{self, cbor::Behaviour},
	swarm::{NetworkBehaviour, SwarmEvent},
	yamux, Multiaddr, StreamProtocol, Swarm, Transport as _,
};
#[cfg(not(target_arch = "wasm32"))]
use libp2p::{mdns, tcp};
use tokio_with_wasm::alias as tokio;

use super::subfield::*;
use std::error::Error;

//use wasm_bindgen::prelude::*;

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SwarmConfig {
	pub keypair: Keypair,
	#[cfg(not(target_arch = "wasm32"))]
	pub listen_addresses: Vec<String>,
	#[cfg(target_arch = "wasm32")]
	pub bootstrap_urls: Vec<String>,
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_swarm(
	swarm_config: SwarmConfig,
) -> Result<Swarm<SubfieldBehaviour>, Box<dyn Error>> {
	use tokio::time::Duration;

	let mut swarm = libp2p::SwarmBuilder::with_existing_identity(
		swarm_config.keypair.clone().into(),
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
	.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))?
	.with_swarm_config(|c| {
		c.with_idle_connection_timeout(Duration::from_secs(60))
	})
	.build();

	for addr in swarm_config.listen_addresses {
		swarm.listen_on(addr.parse::<Multiaddr>()?)?;
	}

	Ok(swarm)
}



#[cfg(target_arch = "wasm32")]
pub async fn create_swarm(
	swarm_config: SwarmConfig,
) -> Result<Swarm<SubfieldBehaviour>, Box<dyn Error>> {
	use std::time::Duration;

	let mut swarm = libp2p::SwarmBuilder::with_existing_identity(
		swarm_config.keypair.clone().into(),
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

	for addr in swarm_config.bootstrap_urls {
		match swarm.dial(
			addr.parse::<Multiaddr>()
				.map_err(|_| "Failed to parse bootstrap URL")?,
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
