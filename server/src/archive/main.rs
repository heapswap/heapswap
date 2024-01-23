use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket}, State, Path},
    response::Response,
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use dashmap::DashMap;
use yrs::{Doc, ReadTxn, StateVector, Text, Transact, Update};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;

#[derive(Clone)]
struct AppState {
    //docs: Arc<Mutex<HashMap<String, String>>>,
	docs: Arc<DashMap<String, Doc>>,
}

async fn handler( ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
	//let room_name = room_name.unwrap_or_else(|| "default".to_string());
	//println!("Got request for room {}", room_name);
	ws.on_upgrade(|socket| handle_socket(String::from("default"), socket, state))
} 

async fn handler_room_name(Path(room_name): Path<String>, ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
	println!("Got request for room {}", room_name);
	ws.on_upgrade(|socket| handle_socket(room_name, socket, state))
} 

async fn handle_socket(room_name: String, mut socket: WebSocket, state: AppState) {
   	
	while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            println!("Got message: {:?}", msg);
			msg
			//return;
        } else {
            // client disconnected
			println!("Client disconnected");
            return;
        };
		
		if let Some(mut doc_entry) = state.docs.get_mut(&room_name) {
			let doc = doc_entry.value_mut();
		
			// Check if the message is binary and decode it
			if let axum::extract::ws::Message::Binary(bin_msg) = msg {
				if let Ok(update) = Update::decode_v1(bin_msg.as_slice()) {
					// Start a new scope for the TransactionMut object
					{
						let mut txn = doc.transact_mut();
						// Apply the update to the document
						txn.apply_update(update);
					} // txn is dropped here
				}
			}
		
			// Encode the document's state and send it back to the client
			{
				let txn = doc.transact();
				let state_vector = txn.state_vector().encode_v1();
				if socket.send(axum::extract::ws::Message::Binary(state_vector)).await.is_err() {
					// client disconnected
					println!("Client disconnected");
					return;
				}
			}
		} else {
			// Handle the case where the room does not exist
		}
    }
}

#[tokio::main]
async fn main() {
	// ...
let router = Router::new()
	.route("/", get(|| async { "Hello, World!" }))
    .route("/ws", get(handler))
	.route("/ws/:roomName", get(handler_room_name))
    .with_state(AppState {
		docs: Arc::new(DashMap::new()),
	});

	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
	
	println!("Listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, router).await.unwrap();
	println!("Bye!");
}