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
pub fn create_swarm(
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
	.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))?
	.with_swarm_config(|c| {
		c.with_idle_connection_timeout(Duration::from_secs(60))
	})
	.build();

	for addr in swarm_config.listen_addresses {
		swarm.listen_on(addr.parse()?)?;
	}

	Ok(swarm)
}

#[cfg(target_arch = "wasm32")]
pub fn create_swarm(
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
			.authenticate(noise::Config::new(&key).unwrap())
			.multiplex(yamux::Config::default())
			.boxed())
	})?
	.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))?
	.with_swarm_config(|c| {
		c.with_idle_connection_timeout(Duration::from_secs(60))
	})
	.build();

	for addr in swarm_config.bootstrap_urls {
		swarm.dial(addr.parse::<Multiaddr>()?)?;
	}

	Ok(swarm)
}
