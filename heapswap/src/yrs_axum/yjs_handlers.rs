use crate::app_state::GlobalAppState;
use crate::yrs_axum::{AxumSink, AxumStream, YrsDoc};
use axum::{
	extract::{
		ws::{WebSocket, WebSocketUpgrade},
		Path, State,
	},
	response::IntoResponse,
};
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

// connect with no room name
pub async fn get_yjs_default_room_handler(
	ws: WebSocketUpgrade,
	State(state): State<GlobalAppState>,
) -> impl IntoResponse {
	// the default room name is hardcoded to "__default"
	let room_name = String::from("__default");
	ws.on_upgrade(|socket| yjs_socket_handler(room_name, socket, state))
}

// connect with a room name
pub async fn get_yjs_named_room_handler(
	ws: WebSocketUpgrade,
	Path(room_name): Path<String>,
	State(state): State<GlobalAppState>,
) -> impl IntoResponse {
	//let room_name = String::from("default");

	println!("Got request for room {}", room_name);
	ws.on_upgrade(|socket| yjs_socket_handler(room_name, socket, state))
}

// handle the socket
// - keep the socket open
// - broadcast updates to all clients
// - receive updates from all clients
async fn yjs_socket_handler(
	room_name: String,
	ws: WebSocket,
	state: GlobalAppState,
) {
	/*
				// pre-initialize code mirror document with some text
				let txt = doc.get_or_insert_text("codemirror");
				let mut txn = doc.transact_mut();
				txt.push(
					&mut txn,
					r#"function hello() {
	  console.log('hello world');
	}"#,
				);

				*/

	// if room does not exist, create it
	if !state.get_docs().contains_key(&room_name) {
		state
			.get_docs()
			.insert(room_name.clone(), YrsDoc::new().await);
	}

	// get the document
	let yrs_doc_item = state.get_docs().get(&room_name).unwrap();
	let yrs_doc = yrs_doc_item.value();

	// create a sink and stream for the websocket
	let (sink, stream) = ws.split();
	let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
	let stream = AxumStream::from(stream);

	// create a broadcast subscription
	//let bsub = yrs_doc.broadcast.subscribe(sink, stream);

	let bsub = yrs_doc.get_broadcast().subscribe(sink, stream);

	// wait for the broadcast to finish
	match bsub.completed().await {
		Ok(_) => println!("broadcasting finished successfully"),
		Err(e) => {
			eprintln!("broadcasting finished abruptly: {}", e)
		}
	}
}
