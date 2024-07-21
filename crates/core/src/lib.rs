#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod misc;
pub use misc::*;

pub mod arr;
pub mod crypto;

use wasm_bindgen::prelude::*;

use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use tracing_wasm::{WASMLayer, WASMLayerConfig};

#[cfg(target_arch = "wasm32")]
// try to set the global default subscriber
pub fn try_set_as_global_default_with_config(
	config: WASMLayerConfig,
) -> Result<(), SetGlobalDefaultError> {
	tracing::subscriber::set_global_default(
		Registry::default().with(WASMLayer::new(config)),
	)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn wasm_start() {
	// set panic hook
	console_error_panic_hook::set_once();

	// set tracing level
	let level = tracing::Level::DEBUG;
	let tracing_cfg = tracing_wasm::WASMLayerConfigBuilder::new()
		.set_max_level(level)
		.build();
	let _ = try_set_as_global_default_with_config(tracing_cfg);

	tracing::info!("initialized with tracing level: {:?}", level);
}
