#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_mut)]

pub use eyre::{
	eyre as eyr, Ok as EOk, OptionExt as _, Report as EReport,
	Result as EResult,
};
/**
 * Reexports
*/
// pub use futures::prelude::*;
pub use lazy_static::lazy_static;
pub use libp2p;
pub use once_cell::sync::{Lazy, OnceCell};
pub use reqwest;
pub use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use bincode::{deserialize, serialize};
// pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
pub use getset::{
	Getters, Setters,
};
pub use std::sync::Arc;
pub use strum;
pub use tracing;

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
 * Exports
*/
pub mod arr;
pub mod crypto;
pub mod subfield;

mod misc;
pub use misc::*;
pub mod constants;
pub use constants::*;
