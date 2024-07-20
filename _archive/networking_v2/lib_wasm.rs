use wasm_bindgen::prelude::*;

use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use tracing_wasm::{WASMLayer, WASMLayerConfig};

// try to set the global default subscriber
pub fn try_set_as_global_default_with_config(
	config: WASMLayerConfig,
) -> Result<(), SetGlobalDefaultError> {
	tracing::subscriber::set_global_default(
		Registry::default().with(WASMLayer::new(config)),
	) 
}

#[wasm_bindgen(start)]
fn wasm_start() {

	let using_bevy = false;

	if !using_bevy {
		// set panic hook
		console_error_panic_hook::set_once();

		// set tracing level
		let level = tracing::Level::DEBUG;
		let tracing_cfg = tracing_wasm::WASMLayerConfigBuilder::new()
			.set_max_level(level)
			.build();
		let _ = try_set_as_global_default_with_config(tracing_cfg);

		tracing::info!("initialized with bevy: {:?} and tracing level: {:?}", using_bevy, level);
	}

	//async_std::task::block_on(async move { 
	//	let _ =crate::bev::entrypoint::main(using_bevy).await;		
	//});
}
