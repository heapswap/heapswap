//! Example websocket server.
//!
//! Run the server with
//! ```not_rust
//! cargo run -p example-websockets --bin example-websockets
//! ```
//!
//! Run a browser client with
//! ```not_rust
//! firefox http://localhost:3000
//! ```
//!
//! Alternatively you can run the rust client (showing two
//! concurrent websocket connections being established) with
//! ```not_rust
//! cargo run -p example-websockets --bin example-client
//! ```
//! 
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]

use axum::{
	extract::ws::{Message, WebSocket, WebSocketUpgrade},
	response::IntoResponse,
	routing::get,
	Router,
};
//use axum_extra::TypedHeader;

use std::borrow::Cow;
use std::ops::ControlFlow;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
	services::ServeDir,
	trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use heapswap_core::{messages::hello::Hello, networking::noise_websocket::{self, NoiseWebsocket}};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};

#[tokio::main]
async fn main() {
	tracing_subscriber::registry()
		.with(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| {
					"example_websockets=debug,tower_http=debug".into()
				}),
		)
		.with(tracing_subscriber::fmt::layer())
		.init();

	let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

	// build our application with some routes
	let app = Router::new()
		.fallback_service(
			ServeDir::new(assets_dir).append_index_html_on_directories(true),
		)
		.route("/ws", get(ws_handler))
		// logging so we can see whats going on
		.layer(
			TraceLayer::new_for_http().make_span_with(
				DefaultMakeSpan::default().include_headers(true),
			),
		);

	// run it with hyper
	let port = 3000;
	let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
		.await
		.unwrap();
	tracing::debug!("listening on {}", listener.local_addr().unwrap());
	axum::serve(
		listener,
		app.into_make_service_with_connect_info::<SocketAddr>(),
	)
	.await
	.unwrap();
}

async fn ws_handler(
	ws: WebSocketUpgrade,
	ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
	ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr){
	
	let mut socket = Box::new(socket);
	
	//let mut socket : reqwest_websocket::WebSocket = socket;
	
	let noise_websocket: NoiseWebsocket::<Hello> = NoiseWebsocket::responder(socket).await.unwrap();
}
