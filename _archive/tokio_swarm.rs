#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]
#![allow(unused_variables)]

use bytes::Bytes;
use futures::{prelude::*, stream::StreamExt};
use heapswap_core::{bys, crypto::keys::KeyPair, networking::subfield::*};
use libp2p::{
	gossipsub,
	kad::Behaviour as KadBehaviour,
	mdns, noise, request_response,
	swarm::{NetworkBehaviour, SwarmEvent},
	tcp, yamux, Swarm,
};
use libp2p::{
	identity::ed25519::Keypair, request_response::ProtocolSupport,
	StreamProtocol,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio::{
	io,
	io::AsyncBufReadExt,
	net::TcpListener,
	select,
	sync::{Mutex, MutexGuard},
};

pub struct SubfieldSwarmConfig {
	//pub keypair: KeyPair,
	pub listen_addresses: Vec<String>,
	pub is_server: bool,
}

pub fn create_tokio_swarm(
	swarm_config: SubfieldSwarmConfig,
) -> Result<Swarm<SubfieldBehaviour>, Box<dyn Error>> {
	//let _keypair: Keypair = Keypair::generate();

	let mut swarm = libp2p::SwarmBuilder::with_new_identity()
		.with_tokio()
		.with_tcp(
			tcp::Config::default(),
			noise::Config::new,
			yamux::Config::default,
		)?
		.with_behaviour(|key| {
			Ok(SubfieldBehaviour::new(key))
			})?
		.with_swarm_config(|c| {
			c.with_idle_connection_timeout(Duration::from_secs(60))
		})
		.build();

	for addr in swarm_config.listen_addresses {
		swarm.listen_on(addr.parse()?)?;
	}

	Ok(swarm)
}
