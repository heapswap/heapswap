#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]
#![allow(unused_variables)]
use axum::extract::State;
use axum::Json;
use futures::StreamExt;
use heapswap_core::{bys, networking::*};
use heapswap_server::networking::*;

use swarm_create::multiaddr::Protocol;
use swarm_create::swarm::handler::multi;
use swarm_create::swarm::SwarmEvent;
use swarm_create::Swarm;
use swarm_create::{
	identity::ed25519::Keypair, request_response::ProtocolSupport,
	StreamProtocol,
};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	// Setup tracing subscriber
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

	let keypair = Keypair::generate();

	let swarm: ThreadsafeSubfieldSwarm = Arc::new(Mutex::new(
		swarm_create(SwarmConfig {
			keypair: keypair.clone().into(),
			listen_addresses: vec![
				"/ip4/0.0.0.0/tcp/0/ws".to_string(),
				"/ip6/::/tcp/0/ws".to_string(),
				//"/ip4/0.0.0.0/ws".to_string(),
			],
		})
		.await?,
	));

	// Create shared state
	let state = AppState {
		swarm: swarm.clone(),
	};

	let app = Router::new()
		.route("/bootstrap", get(get_bootstrap))
		.with_state(state);

	let axum_handle = spawn_axum_loop(app, 3000);

	let swarm_handle = spawn_swarm_loop(swarm.clone());

	let _ = tokio::try_join!(axum_handle, swarm_handle).map(|_| ());

	Ok(())
}
