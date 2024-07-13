
use std::{borrow::Borrow, net::{Ipv4Addr, Ipv6Addr}, sync::Arc};
use futures::prelude::*;
use futures::{stream::{SplitSink, SplitStream}, AsyncRead, AsyncWrite, SinkExt, StreamExt};
use futures::task::{Spawn, SpawnExt};
use futures::{future, stream, AsyncReadExt, AsyncWriteExt, FutureExt};
use gloo::{
	net::websocket::{events::CloseEvent, futures::WebSocket, Message},
	timers::callback::Timeout,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::window;
use web_sys::Event;
use crate::crypto::{noise::*, keys::*};
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::{Buf, BufMut, BytesMut};
use tokio::sync::{Mutex, MutexGuard};
use super::websocket::*;

pub struct Address {
	public_key: PublicKey,
	ipv4_addr: Option<Ipv4Addr>,
	ipv4_port: Option<u16>,
	ipv6_addr: Option<Ipv6Addr>,
	ipv6_port: Option<u16>
}

#[derive(Debug)]
pub enum ConnectionError {
	InvalidWebsocketObject,
	WebSocketError,
	NoiseError
}


#[derive(Debug, PartialEq)]
pub enum ConnectionRole {
	Client,
	Server
}


pub struct Connection {
	role: ConnectionRole,
	noise: Noise,
	yamux: yamux::Connection<WebSocketWrapper>,
	//ws: WebSocketWrapper
}





impl Connection {
	
	fn new(role: ConnectionRole, noise: Noise, _yamux: yamux::Connection<WebSocketWrapper>	) -> Result<Self, ConnectionError> {
		
		Ok(Self {
			role,
			noise,
			yamux: _yamux
		})
	}

	
	pub fn client(keypair: Keypair, ws: web_sys::WebSocket) -> Result<Self, ConnectionError> {
		let ws = WebSocketWrapper::new(ws);
		let noise = Noise::initiator_from_keypair(keypair);
		let yamux = yamux::Connection::new(ws, yamux::Config::default(), yamux::Mode::Client);
		
		Self::new(ConnectionRole::Client, noise, yamux)
	}
	
	pub fn server(keypair: Keypair, ws: web_sys::WebSocket) -> Result<Self, ConnectionError>{
		let ws = WebSocketWrapper::new(ws);
		let noise = Noise::responder_from_keypair(keypair);
		let yamux = yamux::Connection::new(ws, yamux::Config::default(), yamux::Mode::Server);
		
		
		Self::new(ConnectionRole::Server, noise, yamux)	
	}	
	
	
	
	
}
	
	