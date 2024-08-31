#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_mut)]
// #![feature(trivial_bounds)]

// pub use bincode::{
// 	deserialize, serialize,
// };
// pub use subfield_proto::*;
// pub use subfield_proto as proto;
// pub use subfield_proto::{proto_serialize, proto_deserialize};
pub use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use eyre::{
	eyre as eyr, Ok as EOk, OptionExt as _, Report as EReport,
	Result as EResult,
};

pub fn cbor_serialize<T: Serialize>(value: &T) -> EResult<Vec<u8>> {
	EOk(cbor4ii::serde::to_vec(Vec::new(), value)?)
}

pub fn cbor_deserialize<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> EResult<T> {
	EOk(cbor4ii::serde::from_slice(bytes)?)
}

pub use chrono::{DateTime, Utc};
pub type DateTimeUtc = DateTime<Utc>;

pub use std::collections::{HashMap, HashSet};

pub use lazy_static::lazy_static;
pub use libp2p::request_response::ResponseChannel;
pub use libp2p::request_response::{InboundRequestId, OutboundRequestId};
pub use libp2p::{PeerId, Stream};
pub use libp2p_stream as stream;

/**
 * Reexports
*/
// pub use futures::prelude::*;
pub use std::fmt;
// pub use libp2p;
pub use getset::{CopyGetters, Getters, MutGetters, Setters};
pub use once_cell::sync::{Lazy, OnceCell};
pub use reqwest;
// pub use prost::Message;
pub use async_trait::async_trait;
pub use dashmap::{DashMap, DashSet};
pub use futures::{AsyncReadExt, AsyncWriteExt, FutureExt, SinkExt, StreamExt};
pub use itertools::Itertools;
pub use js_sys::Uint8Array;
pub use libp2p;
pub use ordered_float::OrderedFloat;
pub use rand::{prelude::*, thread_rng, Rng};
pub use serde::{
	de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer,
};
pub use std::pin::Pin;
pub use std::sync::Arc;
pub use strum;
pub use tracing;
pub use wasm_bindgen::prelude::*;

/**
 * Channel Reexports
*/

#[cfg(not(any(feature = "server", test)))]
pub use futures::channel::{
	mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
	oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender},
};

#[cfg(any(feature = "server", test))]
pub use {
	// tokio_stream::StreamExt as _,
	tokio::sync::{
		mpsc::UnboundedSender,
		oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender},
	},
	tokio_stream::wrappers::UnboundedReceiverStream as UnboundedReceiver,
};

#[cfg(any(feature = "server", test))]
pub fn unbounded<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
	let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<T>();
	let rx = UnboundedReceiver::new(rx);
	(tx, rx)
}

/**
 * Mutex Reexports
*/
#[cfg(not(feature = "server"))]
pub use {
	// std::sync::{Mutex, MutexGuard},
	futures::lock::{MappedMutexGuard, Mutex, MutexGuard},
	std::sync::RwLock, // std::thread::yield_now,
};
#[cfg(feature = "server")]
pub use {
	tokio,
	tokio::sync::{MappedMutexGuard, Mutex, MutexGuard, RwLock},
	tokio::task::yield_now,
};

/**
 * Prelude
*/
pub mod constants;
pub mod crypto;
pub mod misc;
pub mod proto;
// pub mod networking;
pub mod store;
pub mod swarm;

pub mod prelude {
	pub use super::constants::*;
	pub use super::crypto::*;
	pub use super::proto::*;
	// pub use super::networking::*;
	pub use super::store::*;
	pub use super::swarm::*;
	pub use crate::misc::*;
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
