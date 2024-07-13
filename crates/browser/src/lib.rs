#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
//#![allow(non_snake_case)]

//pub use wasm_bindgen_rayon::init_thread_pool;

mod misc;
pub use misc::*;
use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use tracing_wasm::{WASMLayer, WASMLayerConfig};
use wasm_bindgen::prelude::*;

pub mod arr;
pub mod crypto;
pub mod jac_dht;
pub mod networking;

// try to set the global default subscriber
pub fn try_set_as_global_default_with_config(
	config: WASMLayerConfig,
) -> Result<(), SetGlobalDefaultError> {
	tracing::subscriber::set_global_default(
		Registry::default().with(WASMLayer::new(config)),
	)
}

#[wasm_bindgen(start)]
fn main() {
	// set panic hook
	console_error_panic_hook::set_once();

	// set tracing level
	let level = tracing::Level::INFO;
	let tracing_cfg = tracing_wasm::WASMLayerConfigBuilder::new()
		.set_max_level(level)
		.build();
	let _ = try_set_as_global_default_with_config(tracing_cfg);

	tracing::info!("tracing initialized with level: {:?}", level);
}
