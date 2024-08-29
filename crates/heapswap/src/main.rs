#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(dead_code)]

use subfield::libp2p::futures::FutureExt;
use subfield::libp2p::multiaddr::{Multiaddr, Protocol};
// use subfield::store::SubfieldStoreConfig;
use std::borrow::Cow;
use subfield::*;

use std::borrow::BorrowMut;
use std::{
	net::{IpAddr, Ipv4Addr, Ipv6Addr},
	sync::Arc,
};

use {
	axum::routing::*,
	axum::{
		debug_handler,
		extract::{Json, State},
		http::StatusCode,
		response::{Html, IntoResponse},
		routing::get,
		Router,
	},
	http::{Method, Request, Response},
	std::net::SocketAddr,
	std::path::PathBuf,
	tokio::select,
	tokio::sync::{Mutex, MutexGuard, RwLock},
	tokio::task::yield_now,
	tokio::{net::TcpListener, time::Duration},
	tower_http::cors::{Any, CorsLayer},
};

#[derive(Clone)]
pub struct AppState {
	// swarm_tx: Transmitter<SubfieldRequest>,
	subfield_client: Arc<subfield::swarm::SubfieldClient>,
}

#[tokio::main]
async fn main() -> EResult<()> {
	main_inner(|_| async move { Ok(()) }).await
}

async fn main_inner<F, Fut>(test_func: F) -> EResult<()>
where
	F: FnOnce(Arc<subfield::swarm::SubfieldClient>) -> Fut + Send + 'static,
	Fut: std::future::Future<Output = EResult<()>> + Send,
{
	// use tracing subscriber
	let _ = tracing::subscriber::set_global_default(
		tracing_subscriber::FmtSubscriber::new(),
	);

	tracing::info!("DEV_MODE: {}", DEV_MODE);

	// try to open port - if unsuccessful, add 1 and retry
	let mut port: u16 = SERVER_PORT;
	let mut try_listener;
	loop {
		try_listener =
			tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await;
		if try_listener.is_ok() {
			break;
		}
		port += 1;
	}
	let listener = try_listener.unwrap();

	// Get local swarm addresses

	let local_ipv4 = IpAddr::from(Ipv4Addr::UNSPECIFIED);
	let local_ipv6 = IpAddr::from(Ipv6Addr::UNSPECIFIED);

	let websocket_ipv4_address = Multiaddr::from(local_ipv4)
		.with(Protocol::Tcp(0))
		.with(Protocol::Ws(Cow::Borrowed("/")));

	let websocket_ipv6_address = Multiaddr::from(local_ipv6)
		.with(Protocol::Tcp(0))
		.with(Protocol::Ws(Cow::Borrowed("/")));

	let webrtc_ipv4_address = Multiaddr::from(local_ipv4)
		.with(Protocol::Udp(0))
		.with(Protocol::WebRTCDirect);

	let webrtc_ipv6_address = Multiaddr::from(local_ipv6)
		.with(Protocol::Udp(0))
		.with(Protocol::WebRTCDirect);

	// let quic_ipv4_address = Multiaddr::from(local_ipv4)
	// 	.with(Protocol::Udp(0))
	// 	.with(Protocol::QuicV1);

	// let quic_ipv6_address = Multiaddr::from(local_ipv6)
	// 	.with(Protocol::Udp(0))
	// 	.with(Protocol::QuicV1);

	// Create keypair

	let keypair = crypto::Keypair::random();

	let config = subfield::swarm::SubfieldConfig {
		listen_addresses: vec![
			websocket_ipv4_address.to_string(),
			websocket_ipv6_address.to_string(),
			// webrtc_ipv4_address.to_string(),
			// webrtc_ipv6_address.to_string(),
		],
		..Default::default()
	};

	let subfield_client =
		Arc::new(subfield::swarm::SubfieldClient::new(config).await?);

	// swarm event loop
	let subfield_client_clone = subfield_client.clone();
	tokio::task::spawn(async move {
		// if port != 3000 {
		// 	subfield_client_clone.bootstrap().await.unwrap();
		// }
		subfield_client_clone.event_loop().await;
	});

	// test func
	let subfield_client_clone2 = subfield_client.clone();
	tokio::task::spawn(async move {
		test_func(subfield_client_clone2).await.unwrap();
	});

	/*
	let subfield_client_clone2 = subfield_client.clone();
	tokio::task::spawn(async move {
		loop {
			let _ =
				tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
			// print connected peers
			let swarm_lock = subfield_client_clone2.swarm().await;
			let peers = swarm_lock
				.connected_peers()
				.map(|peer| peer.to_string())
				.collect::<Vec<String>>();
			drop(swarm_lock);
			tracing::info!("Connected peers: {:?}", peers);

			if peers.len() > 0 {
				 let res = subfield_client_clone2.echo(proto::EchoRequest { message: "hello".to_string() }).await;
				 tracing::info!("Successfully recieved Echo response: {:?}", res);
			 }

		}
	});


	let subfield_client_clone3 = subfield_client.clone();
	let monitor_lock = false;
	if monitor_lock {
		tokio::task::spawn(async move {
			loop {
				// print whether the swarm is locked
				let swarm_lock = subfield_client_clone3.swarm().now_or_never();
				tracing::info!("Swarm locked: {:?}", swarm_lock.is_none());
				drop(swarm_lock);
				tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
			}

		});
	}
	*/

	// App Router

	let state = AppState { subfield_client };

	#[allow(unused_mut)]
	let mut app = Router::new()
		.route("/", get(homepage))
		.route("/bootstrap", get(get_bootstrap))
		.route("/peers", get(get_peers))
		.with_state(state);

	// add cors layer for debug mode
	#[cfg(debug_assertions)]
	{
		let cors = CorsLayer::new()
			// allow `GET` and `POST` when accessing the resource
			.allow_methods([
				Method::GET,
				Method::POST,
				Method::OPTIONS,
				Method::PUT,
				Method::DELETE,
			])
			// allow requests from any origin
			.allow_origin(Any);

		app = app.layer(cors);
	}

	tracing::info!("Listening on {}", listener.local_addr().unwrap());

	axum::serve(listener, app.into_make_service())
		.await
		.unwrap();

	// EOk(())
	// });

	tracing::error!("Axum runtime exited");

	Ok(())
}

async fn homepage() -> Html<&'static str> {
	Html("<h1>Heapswap</h1>")
}

#[debug_handler]
async fn get_bootstrap(State(state): State<AppState>) -> Json<Vec<String>> {
	let filter_private = !DEV_MODE;

	let swarm_lock = state.subfield_client.swarm_lock().await;
	let addresses = swarm_lock
		.listeners()
		.into_iter()
		.filter(|addr| {
			// Check if the address is not local
			!addr.iter().any(|proto| match proto {
				Protocol::Ip4(ip) => {
					(ip.is_loopback() || ip.is_private()) && filter_private
				}
				Protocol::Ip6(ip) => {
					(ip.is_loopback() || ip.is_unspecified()) && filter_private
				}
				_ => false,
			})
		})
		.map(|addr| addr.to_string())
		.collect::<Vec<_>>();

	Json(addresses)
}

async fn get_peers(State(state): State<AppState>) -> Json<Vec<String>> {
	let swarm_lock = state.subfield_client.swarm_lock().await;

	let peers = swarm_lock
		.connected_peers()
		.into_iter()
		.map(|peer| peer.to_string())
		.collect::<Vec<_>>();

	Json(peers)
}

#[tokio::test]
async fn test_main() -> EResult<()> {
	let mut handles = vec![];
	for i in 0..3 {
		let handle = tokio::spawn(async move {
			main_inner(|swarm_client| async move {
				loop {
					let _ =
						tokio::time::sleep(tokio::time::Duration::from_secs(5))
							.await;
					// print connected peers
					let swarm_lock = swarm_client.swarm_lock().await;
					let peers = swarm_lock
						.connected_peers()
						.map(|peer| peer.to_string())
						.collect::<Vec<String>>();
					drop(swarm_lock);
					tracing::info!("Connected peers: {:?}", peers);

					if !peers.is_empty() {
						let res = swarm_client
							.echo(RoutingSubkey::random(), "hello")
							.await;
						tracing::info!(
							"Successfully received Echo response: {:?}",
							res
						);
					}
				}
			})
			.await
		});
		handles.push(handle);
	}
	for handle in handles {
		handle.await??;
	}
	Ok(())
}
