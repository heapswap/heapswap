#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]


/*
   Prelude
*/
mod constants;
mod crypto;
mod misc;
mod protocol;
mod dht;

// mod chord;
// mod store;
// mod swarm;
// #[cfg(feature = "client")]
// mod client;
// #[cfg(feature = "server")]
// mod server;

pub mod prelude {
	pub use crate::constants::*;
	pub use crate::crypto::*;
	pub use crate::misc::*;
	pub use crate::protocol::*;
	pub use crate::dht::*;
	// pub use crate::chord::*;
	// pub use crate::store::*;
	// pub use crate::swarm::*;
	// #[cfg(feature = "client")]
	// pub use crate::client::*;
	// #[cfg(feature = "server")]
	// pub use crate::server::*;
}
pub use crate::prelude::*;

// tests
#[cfg(test)]
pub mod tests;

/*
   Reexports
*/
pub use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use chrono::{DateTime, Utc};
pub use eyre::{
	eyre as eyr, Ok as EOk, OptionExt as _, Report as EReport,
	Result as EResult,
};
pub use std::borrow::Cow;
pub type DateTimeUtc = DateTime<Utc>;
pub use async_trait::async_trait;
pub use dashmap::{DashMap, DashSet};
pub use either::Either;
pub use futures::prelude::*;
pub use futures::{AsyncReadExt, AsyncWriteExt, FutureExt, SinkExt, StreamExt};
pub use getset::{CopyGetters, Getters, MutGetters, Setters};
pub use itertools::Itertools;
pub use js_sys::Uint8Array;
pub use lazy_static::lazy_static;
pub use once_cell::sync::{Lazy, OnceCell};
pub use ordered_float::OrderedFloat;
pub use rand::{prelude::*, random, thread_rng, Rng};
// pub use reqwest;
pub use serde::{
	de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer,
};
pub use std::collections::{HashMap, HashSet};
pub use std::fmt;
pub use std::pin::Pin;
pub use std::sync::Arc;
pub use strum;
pub use tracing;
pub use wasm_bindgen::prelude::*;

/*
   Channel Reexports
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

/*
   Mutex Reexports
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

/*
   WASM Setup
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

/*
   WASM Entrypoint
*/
#[wasm_bindgen(start)]
pub async fn main() {
	// set tracing level
	console_error_panic_hook::set_once();
	let level = tracing::Level::INFO;
	let tracing_cfg = tracing_wasm::WASMLayerConfigBuilder::new()
		.set_max_level(level)
		.build();
	let _ = try_set_as_global_default_with_config(tracing_cfg);
}

/*
 Libp2p
*/
pub use bincode::{deserialize, serialize};

// pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, SubfieldError> {
// 	cbor4ii::serde::to_vec(Vec::new(), value)
// 		.map_err(|e| SubfieldError::SerializationFailed)
// }

// pub fn deserialize<'a, T: Deserialize<'a>>(
// 	bytes: &'a [u8],
// ) -> Result<T, SubfieldError> {
// 	cbor4ii::serde::from_slice(bytes)
// 		.map_err(|e| SubfieldError::DeserializationFailed)
// }

pub use libp2p;
// // pub use libp2p::request_response::ResponseChannel;
// // pub use libp2p::request_response::{InboundRequestId, OutboundRequestId};
pub use libp2p::{Multiaddr, PeerId, Stream};
// pub use libp2p_stream as stream;
