use futures::{SinkExt, StreamExt};
use gloo::{
	net::websocket::{events::CloseEvent, futures::WebSocket, Message},
	timers::callback::Timeout,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::window;
use web_sys::Event;

#[wasm_bindgen]
pub fn echo() {
	let ws = WebSocket::open("wss://echo.websocket.org").unwrap();
	let (mut write, 
		mut read) = ws.split();

	spawn_local(async move {
		write
			.send(Message::Text(String::from("test")))
			.await
			.unwrap();
		write
			.send(Message::Text(String::from("test 2")))
			.await
			.unwrap();
		
		write.close().await.unwrap();
	});
	

	spawn_local(async move {
		while let Some(msg) = read.next().await {
			tracing::info!("{:?}", msg)
		}

		tracing::info!("WebSocket Closed");
	})
}


#[wasm_bindgen]
pub fn echo_ws(ws: JsValue) {
	// convert the passed in JsValue to a gloo WebSocket
	let ws: web_sys::WebSocket = ws.dyn_into::<web_sys::WebSocket>().expect("Unable to cast to WebSocket");
	let ws: WebSocket = WebSocket::try_from(ws).expect("Unable to cast to WebSocket");
	
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
		
		write.close().await.unwrap();
	});

	spawn_local(async move {
		while let Some(msg) = read.next().await {
			tracing::info!("{:?}", msg)
		}

		tracing::info!("WebSocket Closed");
	})
}


fn emit(event_name: &str) {
    // Get the window object as an event target
    let window = window().unwrap();
    let target = window.dyn_ref::<web_sys::EventTarget>().unwrap();

    // Create a new custom event
    let event = Event::new("event_name").unwrap();

    // Dispatch the event on the window
    target.dispatch_event(&event).unwrap();
}