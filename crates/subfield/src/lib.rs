#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_mut)]

pub use bincode::{deserialize as bincode_deserialize, serialize as bincode_serialize};
// pub use subfield_proto::*;
pub use subfield_proto as proto;
// pub use subfield_proto::{proto_serialize, proto_deserialize};
pub use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use eyre::{
	eyre as eyr, Ok as EOk, OptionExt as _, Report as EReport,
	Result as EResult,
};
/**
 * Reexports
*/
// pub use futures::prelude::*;
pub use lazy_static::lazy_static;
// pub use libp2p;
pub use getset::{CopyGetters, Getters, MutGetters, Setters};
pub use once_cell::sync::{Lazy, OnceCell};
pub use reqwest;
// pub use prost::Message;
pub use dashmap::{DashMap, DashSet};
pub use futures::{AsyncReadExt, AsyncWriteExt, FutureExt, SinkExt, StreamExt};
pub use itertools::Itertools;
pub use js_sys::Uint8Array;
pub use ordered_float::OrderedFloat;
pub use rand::{thread_rng, Rng};
pub use serde::{
	de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer,
};
pub use std::sync::Arc;
pub use strum;
pub use tracing;
pub use wasm_bindgen::prelude::*;

// Mutex reexport
#[cfg(not(feature = "server"))]
pub use {
	// std::sync::{Mutex, MutexGuard},
	futures::lock::{Mutex, MutexGuard},
	std::sync::RwLock, // std::thread::yield_now,
};
#[cfg(feature = "server")]
pub use {
	tokio,
	tokio::sync::{Mutex, MutexGuard, RwLock},
	tokio::task::yield_now,
};

/**
 * Prelude
*/
pub mod misc;
pub mod constants;
pub mod crypto;
pub mod store;
pub mod protocol;
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;


pub mod prelude {
    pub use crate::misc::*;
    pub use super::constants::*;
    pub use super::crypto::*;
    pub use super::store::*;
    pub use super::protocol::*;
    #[cfg(feature = "client")]
    pub use super::client::*;
    #[cfg(feature = "server")]
    pub use super::server::*;
}
pub use crate::prelude::*;

// tests
#[cfg(test)]
pub mod tests;


/**
 * WASM Setup
*/
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

/**
 * WASM Entrypoint
*/
#[wasm_bindgen(start)]
pub fn start() {
	// set tracing level
	console_error_panic_hook::set_once();
	let level = tracing::Level::INFO;
	let tracing_cfg = tracing_wasm::WASMLayerConfigBuilder::new()
		.set_max_level(level)
		.build();
	let _ = try_set_as_global_default_with_config(tracing_cfg);
}

#[wasm_bindgen(js_name = "hello")]
pub fn hello() -> String {
	"hello".to_string()
}
