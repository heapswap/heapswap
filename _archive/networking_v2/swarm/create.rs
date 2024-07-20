use futures::{prelude::*, stream::StreamExt};
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;

use crate::subfield::*;
use crate::{arr, crypto::keys::Keypair};
use bytes::Bytes;
use libp2p::{
	core::muxing::StreamMuxerBox,
	gossipsub,
	kad::QueryResult as KadQueryResult,
	noise, ping,
	swarm::{NetworkBehaviour, SwarmEvent},
	yamux, Multiaddr, Swarm, Transport,
};
#[cfg(not(target_arch = "wasm32"))]
use libp2p::{mdns, quic, tcp};
#[cfg(not(target_arch = "wasm32"))]
use libp2p_webrtc as webrtc;
#[cfg(target_arch = "wasm32")]
use libp2p::webtransport_websys;
//use libp2p_webrtc_websys as webrtc_websys;
//#[cfg(not(target_arch = "wasm32"))]
use crate::subfield::*;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[cfg(target_arch = "wasm32")]
use std::sync::{Mutex, MutexGuard};

//pub type SubfieldSwarm = Arc<Mutex<Swarm<SubfieldBehaviour>>>;
//pub type SubfieldSwarmEvent = SwarmEvent<SubfieldBehaviourEvent>;
pub type SubfieldSwarm = Swarm<SubfieldBehaviour>;
pub type SubfieldSwarmEvent = SwarmEvent<SubfieldBehaviourEvent>;

#[derive(Debug)]
pub enum SubfieldCreateSwarmError {
	FailedToLockSwarm,
}

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
	pub listen_addresses: Vec<String>,
	pub bootstrap_multiaddrs: Vec<String>,
}

/**
 * Wasm
*/
#[cfg(target_arch = "wasm32")]
pub async fn swarm_create(
	swarm_config: SwarmConfig,
) -> Result<SubfieldSwarm, Box<dyn Error>> {
	use libp2p::core::upgrade;
	use std::time::Duration;
	
	tracing::info!("Creating Swarm");

	let mut swarm = libp2p::SwarmBuilder::with_existing_identity(
		heapswap_keypair_to_libp2p_keypair(&swarm_config.keypair),
	)
	.with_wasm_bindgen()
	.with_other_transport(|key| {
		let config = webtransport_websys::Config::new(&key);
		let transport = webtransport_websys::Transport::new(config).boxed();
		Ok(transport)
	})?
	.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))?
	.with_swarm_config(|c| {
		c.with_idle_connection_timeout(Duration::from_secs(60))
	})
	.build();

	tracing::info!("Successfully created swarm");

	for addr in swarm_config.bootstrap_multiaddrs {
		tracing::info!("Dialing bootstrap URL: {:?}", addr);
		match swarm.dial(
			addr.parse::<Multiaddr>()
				.map_err(|_| "Failed to parse bootstrap multiaddr")?,
		) {
			Ok(_) => {
				tracing::info!(
					"Successfully dialed bootstrap URL: {:?}",
					addr
				);
				return Ok(swarm)
			}
			//Err(e) => eprintln!("Failed to dial bootstrap URL: {:?}", e),
			Err(_) => {}
		}
	}

	//Ok(Arc::new(Mutex::new(swarm)))
	//Ok(swarm)
	panic!("No bootstrap multiaddrs successfully dialed");
}

/**
 * Non-Wasm
*/
#[cfg(not(target_arch = "wasm32"))]
pub async fn swarm_create(
	swarm_config: SwarmConfig,
) -> Result<SubfieldSwarm, Box<dyn Error>> {
	let keypair = heapswap_keypair_to_libp2p_keypair(&swarm_config.keypair);

	//let local_cert_path = "./certs/webrtc.pem";
	//let webrtc_cert = read_or_create_certificate(Path::new(local_cert_path))
	//.await
	//.context("Failed to read certificate")?;

	//let transport = {
	//    let webrtc = webrtc::async_std::Transport::new(local_key.clone(), certificate);
	//    let quic = quic::async_std::Transport::new(quic::Config::new(&keypair));

	//    let mapped = webrtc.or_transport(quic).map(|fut, _| match fut {
	//        Either::Right((local_peer_id, conn)) => (local_peer_id, StreamMuxerBox::new(conn)),
	//        Either::Left((local_peer_id, conn)) => (local_peer_id, StreamMuxerBox::new(conn)),
	//    });

	//    dns::AsyncStdDnsConfig::system(mapped)?.boxed()
	//};

	let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
		//.with_async_std()
		.with_tokio()
		//.with_tcp(
		//	tcp::Config::default(),
		//	noise::Config::new,
		//	yamux::Config::default,
		//)?
		//.with_websocket(
		//	(libp2p::tls::Config::new, libp2p::noise::Config::new),
		//	libp2p::yamux::Config::default,
		//).await?
		//.with_other_transport(|id_keys| {
		//	Ok(webrtc::tokio::Transport::new(
		//		id_keys.clone(),
		//		webrtc::tokio::Certificate::generate(&mut rand::thread_rng())?,
		//	)
		//	.map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn))))
		//})?
		.with_quic()
		.with_behaviour(|key| Ok(SubfieldBehaviour::new(key)))
		.map_err(|e| e.to_string())?
		.with_swarm_config(|c| {
			c.with_idle_connection_timeout(Duration::from_secs(60))
		})
		.build();

	for addr in swarm_config.listen_addresses {
		swarm.listen_on(addr.parse::<Multiaddr>()?)?;
	}

	//Ok(Arc::new(Mutex::new(swarm)))
	Ok(swarm)
}

/*
#[cfg(not(target_arch = "wasm32"))]
async fn read_or_create_certificate(path: &Path) -> Result<Certificate> {
	if path.exists() {
		let pem = fs::read_to_string(&path).await?;

		info!("Using existing certificate from {}", path.display());

		return Ok(Certificate::from_pem(&pem)?);
	}

	let cert = Certificate::generate(&mut rand::thread_rng())?;
	fs::write(&path, &cert.serialize_pem().as_bytes()).await?;

	info!(
		"Generated new certificate and wrote it to {}",
		path.display()
	);

	Ok(cert)
}
*/

	//.with_other_transport(|key| {
	//	Ok(webrtc_websys::Transport::new(webrtc_websys::Config::new(&key)))
	//})?	
	/*
	.with_other_transport(|key| {
		let transport = libp2p_websocket_websys::Transport::default()
			.upgrade(libp2p::core::upgrade::Version::V1)
			.authenticate( 
				noise::Config::new(&key)?
			)
			.multiplex(yamux::Config::default())
			.boxed(); 
		Ok(transport)		
	})?
	*/