#![cfg(target_arch = "wasm32")]
#![allow(unused_imports)]
use futures::StreamExt;
use heapswap_core::networking::*;
use lazy_static::lazy_static;
use libp2p::core::Multiaddr;
use libp2p::identity::Keypair;
use libp2p::request_response;
use libp2p::request_response::ProtocolSupport;
use libp2p::swarm::SwarmEvent;
use libp2p::{StreamProtocol, Swarm};
use once_cell::sync::OnceCell;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_with_wasm::alias as tokio;
use tracing::Level;
use wasm_bindgen::{prelude::*, JsError};
use web_sys::{Document, HtmlElement};

static CONFIG: OnceCell<Mutex<Option<SwarmConfig>>> = OnceCell::new();

#[wasm_bindgen]
#[derive(Clone)]
pub struct Config {
	bootstrap_urls: Vec<String>,
}


#[wasm_bindgen]
impl Config {
	#[wasm_bindgen(constructor)]
	pub fn new(bootstrap_urls: Vec<String>) -> Self {
		Self { bootstrap_urls }
	}
}



#[wasm_bindgen]
pub async fn initialize(config: Config) {
	let swarm_config = SwarmConfig {
		bootstrap_urls: config.bootstrap_urls,
		keypair: Keypair::generate_ed25519().into(),
	};

	CONFIG.get_or_init(|| Mutex::new(Some(swarm_config)));
}

#[cfg_attr(target_family = "wasm", wasm_bindgen(start))]
#[cfg_attr(not(target_family = "wasm"), tokio::main(flavor = "current_thread"))]
pub async fn main() -> Result<(), JsError> {
	let config = CONFIG
		.get()
		.ok_or(JsError::new("Config not initialized"))?
		.lock()
		.await
		.clone()
		.ok_or(JsError::new("Config not initialized"))?;

	//let keypair = Keypair::generate_ed25519();

	let swarm: ThreadsafeSubfieldSwarm =
		Arc::new(Mutex::new(create_swarm(config).map_err(|err| {
			std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
		})?));

	let swarm_handle = spawn_swarm_loop(swarm.clone());

	let _ = tokio::try_join!(swarm_handle).map(|_| ());

	Ok(())
}
