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
	// use tracing subscriber
	tracing::subscriber::set_global_default(
		tracing_subscriber::FmtSubscriber::new(),
	)?;

	tracing::info!("DEV_MODE: {}", DEV_MODE);

	// #[cfg(feature = "server")]
	// let _ = tokio::runtime::Runtime::new()
	// 	.unwrap()
	// 	.block_on(async move {
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

	// let swarm_instance: swarm::SubfieldSwarm =
	// // Arc::new(Mutex::new(
	// swarm::create(swarm_config)
	// .await
	// .map_err(|e| eyr!(e.to_string()))?;
	// // ));

	/*
	let threadsafe_swarm: swarm::ThreadsafeSubfieldSwarm =
		Arc::new(Mutex::new(swarm_instance));

	// let (outgoing_swarm_tx, outgoing_swarm_rx) = swarm::new_subfield_message_channel();
	// let (incoming_swarm_tx, incoming_swarm_rx) = swarm::new_subfield_message_channel();

	// Handle swarm events
	// let swarm_instance_cloned = swarm_instance.clone();
	// let mp_swarm = Arc::new(Mutex::new(swarm::MultiplexedSwarm::new(swarm_instance)));
	// let mp_swarm_cloned = mp_swarm.clone();

	let (mut tx, mut rx) = portal::<SubfieldRequest>();
	let swarm_tx = tx.clone();

	let threadsafe_swarm_cloned = threadsafe_swarm.clone();

	let store_threadsafe = Arc::new(Mutex::new(
		store::SubfieldStore::new(SubfieldStoreConfig{
			location: "".to_string(),
		}).await.unwrap(),
	));

	tokio::task::spawn_local(async move {
		loop {
			let mut swarm_lock = threadsafe_swarm_cloned.lock().await;

			// swarm::handle_events(&mut *swarm_lock, &mut incoming_swarm_tx.clone(), &mut outgoing_swarm_rx).await;

			let mut store_lock = store_threadsafe.lock().await;

			let _ = events::handle_events(
				&mut *store_lock,
				&mut *swarm_lock,
				&mut rx,
				&mut tx,
			)
			.now_or_never();

			drop(swarm_lock);
			drop(store_lock);

			tokio::task::yield_now().await;
		}
	});

	*/

	let subfield_client_clone = subfield_client.clone();
	tokio::task::spawn(async move {
		if port != 3000 {
			subfield_client_clone.bootstrap().await.unwrap();
		}
		subfield_client_clone.event_loop().await;
	});

	let subfield_client_clone2 = subfield_client.clone();
	tokio::task::spawn(async move {
		// let _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
		// let message = V256::random256().to_string();
		// let res = subfield_client_clone2
		// 	.echo(proto::EchoRequest { message: message.clone() })
		// 	.await;
		// match res {
		// 	Ok(response) => {
		// 		if response.message != message {
		// 			tracing::error!("ECHO FAILED: {} != {}", message, response.message);
		// 		} else {
		// 			tracing::info!("ECHO SUCCESS: {} == {}", message, response.message);
		// 		}
		// 	}
		// 	Err(e) => {
		// 		tracing::error!("ECHO ERROR: {:?}", e);
		// 	}
		// }
		loop {
			let _ =
				tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
			// print connected peers
			let swarm_lock = subfield_client_clone2.swarm().await;
			let peers = swarm_lock
				.connected_peers()
				.map(|peer| peer.to_string())
				.collect::<Vec<String>>();
			tracing::info!("Connected peers: {:?}", peers);

			// if peers.len() > 0 {
			// 	let _ = subfield_client_clone2.echo(proto::EchoRequest { message: "hello".to_string() }).await;
			// 	// tracing::info!("Echo response: {:?}", res);
			// }

			drop(swarm_lock);
		}
	});

	// App Router

	let state = AppState { subfield_client };

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

	let swarm_lock = state.subfield_client.swarm().await;
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
	let swarm_lock = state.subfield_client.swarm().await;

	let peers = swarm_lock
		.connected_peers()
		.into_iter()
		.map(|peer| peer.to_string())
		.collect::<Vec<_>>();

	Json(peers)
}
