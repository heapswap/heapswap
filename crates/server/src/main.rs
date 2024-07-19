#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]
#![allow(unused_variables)]
use axum::extract::State;
use axum::Json;
use futures::StreamExt;
//use heapswap_core::{bys, networking::*};
use heapswap_server::networking::*;

use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
//use swarm_create::multiaddr::Protocol;
//use swarm_create::swarm::handler::multi;
//use swarm_create::swarm::SwarmEvent;
//use swarm_create::Swarm;  
//use swarm_create::{
//	identity::ed25519::Keypair, request_response::ProtocolSupport,
//	StreamProtocol,
//};
use tokio::net::TcpListener;

use std::error::Error;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::ops::Sub;
use std::sync::Arc;

use futures::task::{Context, Poll, Waker};
use futures::Stream;
use std::future::Future;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::pin::Pin;
use tokio::select;
use tokio::sync::{Mutex, MutexGuard, RwLock};
use tokio::task::yield_now;
use tokio::time::Duration;

use axum::{http::StatusCode, response::Html, routing::get, Router};
use heapswap_core::{
	arr, subfield::*, swarm::*, crypto::*
};

type ThreadsafeSubfieldSwarm = Arc<Mutex<SubfieldSwarm>>;

#[derive(Clone)]
struct AppState {
	swarm: ThreadsafeSubfieldSwarm,
}
 
async fn get_bootstrap(State(state): State<AppState>) -> Json<Vec<String>> {
	let multiaddrs = state
		.swarm
		.lock()
		.await
		.listeners()
		.into_iter()
		.filter(|addr| {
			// Check if the address is not local
			!addr.iter().any(|proto| match proto {
				Protocol::Ip4(ip) => ip.is_loopback() || ip.is_private(),
				Protocol::Ip6(ip) => {
					ip.is_loopback()
						|| ip.segments().starts_with(&[0xfd00, 0x0])
				}
				_ => false,
			})
		})
		.map(|addr| addr.to_string())
		.collect::<Vec<_>>();

	Json(multiaddrs)
}

async fn get_peers(State(state): State<AppState>) -> Json<Vec<String>> {
	
	let swarm = state.swarm.lock().await;
	
    let peers = swarm
        .connected_peers()
        .into_iter()
        .map(|peer_id| {
			peer_id.to_string()
        })
        .collect::<Vec<_>>();

    Json(peers)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	// Setup tracing subscriber
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

	let keypair = keys::Keypair::random();
	
	// random u16 listen address
	//let webrtc_port: u16 = rand::random::<u16>();
	//let quic_port: u16 = rand::random::<u16>();
	
	//let local_ip = IpAddr::from("0.0.0.0");
	
	//let address_webrtc = Multiaddr::from(listen_address)
	//.with(Protocol::Udp(webrtc_port))
	//.with(Protocol::WebRTCDirect);

	//let address_quic = Multiaddr::from(listen_address)
	//	.with(Protocol::Udp(quic_port))
	//	.with(Protocol::QuicV1);

	let swarm: ThreadsafeSubfieldSwarm = Arc::new(Mutex::new(
		swarm_create(SwarmConfig {
			keypair: keypair.clone().into(),
			listen_addresses: vec![
				"/ip4/0.0.0.0/tcp/0/ws".to_string(),
				"/ip6/::/tcp/0/ws".to_string(),
				//"/ip4/0.0.0.0/udp/0/quic".to_string(),
				//"/ip6/::/udp/0/quic".to_string(),
			],
			bootstrap_multiaddrs: vec![]
		})
		.await?,
	));

	// Create shared state
	let state = AppState {
		swarm: swarm.clone(),
	};

	let app = Router::new()
		.route("/", get(get_peers))
		.route("/bootstrap", get(get_bootstrap))
		.route("/peers", get(get_peers))
		.with_state(state);

	let axum_handle = spawn_axum_loop(app, 3000);

	let swarm_handle = spawn_swarm_loop(swarm.clone());
	
	let _ = tokio::try_join!(
		axum_handle, 
		swarm_handle,
	).map(|_| ());

	Ok(())
}

// due to the swarm needing to be wrapped in a mutex for use with axum,
// we need to poll the swarm instead of using the swarm.next() method
async fn poll_swarm(
	swarm: &mut SubfieldSwarm,
) -> Option<SubfieldSwarmEvent> {
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
				let _ =	swarm_handle_event(&mut *lock, event).await;
			}

			let _ = yield_now().await;
		}
	})
}
