#![cfg(target_arch = "wasm32")]
#![allow(unused_imports)]
use futures::StreamExt;
use futures::TryFutureExt;
use heapswap_core::networking::*;
use lazy_static::lazy_static;
use libp2p::core::Multiaddr;
use libp2p::identity::Keypair;
use libp2p::request_response;
use libp2p::request_response::ProtocolSupport;
use libp2p::swarm::SwarmEvent;
use libp2p::{StreamProtocol, Swarm};
use once_cell::sync::OnceCell;
use std::fmt::Display;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_with_wasm::alias as tokio;
use tracing::Level;
use wasm_bindgen::{prelude::*, JsError};
use web_sys::{Document, HtmlElement};

static CONFIG: OnceCell<Mutex<SubfieldSwarmConfig>> = OnceCell::new();
static SWARM: OnceCell<ThreadsafeSubfieldSwarm> = OnceCell::new();

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Config {
	pub bootstrap_urls: Vec<String>,
}

#[wasm_bindgen]
impl Config {
	#[wasm_bindgen(constructor)]
	pub fn new(bootstrap_urls: Vec<String>) -> Self {
		Self { bootstrap_urls }
	}
}

#[wasm_bindgen]
pub async fn init_logging() {
	let level = Level::INFO;
	//let level = match log_level.as_str() {
	//    "Error" => tracing::Level::ERROR,
	//    "Debug" => tracing::Level::DEBUG,
	//    "Trace" => tracing::Level::TRACE,
	//    _ => return Err(JsError::new("Invalid log level")),
	//};

	let tracing_cfg = tracing_wasm::WASMLayerConfigBuilder::new()
		.set_max_level(level)
		.build();

	tracing_wasm::set_as_global_default_with_config(tracing_cfg);
}

#[wasm_bindgen]
pub async fn initialize(config: Config) -> Result<(), JsError> {
	let swarm_config = SubfieldSwarmConfig {
		bootstrap_urls: config.bootstrap_urls,
		keypair: Keypair::generate_ed25519().into(),
	};

	let _ = CONFIG.set(Mutex::new(swarm_config)).ok();
		
	let initalized_config = CONFIG
		.get()
		.ok_or(JsError::new("Config failed to initialize"))?
		.lock()
		.await
		.clone();

	tracing::info!("updated config: {:?}", &initalized_config.bootstrap_urls);

	Ok(())
}

#[cfg_attr(target_family = "wasm", wasm_bindgen(start))]
#[cfg_attr(not(target_family = "wasm"), tokio::main(flavor = "current_thread"))]
pub async fn main() -> Result<(), JsError> {
	Ok(())
}

#[wasm_bindgen]
pub async fn connect() -> Result<(), JsError> {
	//#[wasm_bindgen]
	//pub async fn main() -> Result<(), JsError> {
	let config = CONFIG
		.get()
		.ok_or(JsError::new("Config not initialized"))?
		.lock()
		.await
		.clone();
		//.ok_or(JsError::new("Config not initialized (outer)"))?;

	//let keypair = Keypair::generate_ed25519();

	let swarm: ThreadsafeSubfieldSwarm = Arc::new(Mutex::new(
		create_swarm(config)
			.map_err(|err| {
				std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
			})
			.await?,
	));

	SWARM.set(swarm.clone()).ok();

	let swarm_handle = spawn_swarm_loop(swarm.clone());

	tracing::info!("Entering main loop (rs)");
	let _ = tokio::try_join!(swarm_handle).map(|_| ());
	tracing::info!("Exiting main loop (rs)");

	Ok(())
}


#[wasm_bindgen]
pub async fn create_unordered_list_of_connected_multiaddrs()-> Result<(), JsError> {
	// using web-sys get the ul with id "connected-addresses" and set it to the connected addresses of the swarm (which exists) every 1s

	let doc = web_sys::window().unwrap().document().unwrap();

	let ul = doc.get_element_by_id("connected-addresses").unwrap();

	let swarm = SWARM.get().ok_or(JsError::new("Swarm not initialized"))?.clone();

	let duration = Duration::from_secs(1);

	loop {

		tokio::time::sleep(duration).await;

		let connected_addresses = swarm.lock().await.connected_peers().map(|peer_id| {
			peer_id.to_base58()
		}).collect::<Vec<String>>();

		let mut inner_html = String::new();

		for addr in connected_addresses {
			inner_html.push_str(&format!("<li>{}</li>", addr));
		}

		ul.set_inner_html(&inner_html);
	}

}