use futures::{SinkExt, StreamExt};
use gloo::{
	net::websocket::{events::CloseEvent, futures::WebSocket, Message},
	timers::callback::Timeout,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub fn echo() {
	let ws = WebSocket::open("wss://echo.websocket.org").unwrap();
	let (mut write, mut read) = ws.split();

	spawn_local(async move {
		write
			.send(Message::Text(String::from("test")))
			.await
			.unwrap();
		write
			.send(Message::Text(String::from("test 2")))
			.await
			.unwrap();
	});

	spawn_local(async move {
		while let Some(msg) = read.next().await {
			tracing::info!("{:?}", msg)
		}

		tracing::info!("WebSocket Closed");
	})
}
