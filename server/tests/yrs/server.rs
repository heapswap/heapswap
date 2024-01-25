use axum::{
	extract::{
		ws::{Message, WebSocket, WebSocketUpgrade},
		Path, State,
	},
	response::{IntoResponse, Response},
	routing::get,
	Router,
};

use axum_extra::TypedHeader;
use futures::Future;
use std::borrow::Cow;
use std::ops::ControlFlow;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
	services::ServeDir,
	trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};
use tokio_stream::{self as stream, Stream};

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use y_sync::awareness::Awareness;
use y_sync::net::BroadcastGroup;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Text, Transact, Update};
use yrs_warp::ws::{WarpSink, WarpStream};
//use yrs_warp::AwarenessRef;
use dashmap::DashMap;
//use anyhow::{Error, Result, anyhow};
type AwarenessRef = Arc<RwLock<Awareness>>;
type BroadcastRef = Arc<BroadcastGroup>;

struct YrsDoc {
	//doc: Doc,
	awareness: AwarenessRef,
	broadcast: BroadcastRef,
}

#[derive(Clone)]
struct AppState {
	//docs: Arc<Mutex<HashMap<String, String>>>,
	docs: Arc<DashMap<String, YrsDoc>>,
}

use futures_util::stream::{SplitSink, SplitStream};
use std::pin::Pin;
use std::task::{Context, Poll};
//use y_sync::sync::Error as ySyncError;

#[repr(transparent)]
#[derive(Debug)]
pub struct AxumSink(SplitSink<WebSocket, Message>);

impl From<SplitSink<WebSocket, Message>> for AxumSink {
	fn from(sink: SplitSink<WebSocket, Message>) -> Self {
		AxumSink(sink)
	}
}

impl futures_util::Sink<Vec<u8>> for AxumSink {
	type Error = y_sync::sync::Error;

	fn poll_ready(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		match Pin::new(&mut self.0).poll_ready(cx) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(Err(e)) => {
				Poll::Ready(Err(y_sync::sync::Error::Other(e.into())))
			}
			Poll::Ready(_) => Poll::Ready(Ok(())),
		}
	}

	fn start_send(
		mut self: Pin<&mut Self>,
		item: Vec<u8>,
	) -> Result<(), Self::Error> {
		if let Err(e) = Pin::new(&mut self.0).start_send(axum::extract::ws::Message::Binary(item))
		{
			Err(y_sync::sync::Error::Other(e.into()))
		} else {
			Ok(())
		}
	}

	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		match Pin::new(&mut self.0).poll_flush(cx) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(Err(e)) => {
				Poll::Ready(Err(y_sync::sync::Error::Other(e.into())))
			}
			Poll::Ready(_) => Poll::Ready(Ok(())),
		}
	}

	fn poll_close(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		match Pin::new(&mut self.0).poll_close(cx) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(Err(e)) => {
				Poll::Ready(Err(y_sync::sync::Error::Other(e.into())))
			}
			Poll::Ready(_) => Poll::Ready(Ok(())),
		}
	}
}

#[derive(Debug)]
pub struct AxumStream(SplitStream<WebSocket>);

impl From<SplitStream<WebSocket>> for AxumStream {
	fn from(stream: SplitStream<WebSocket>) -> Self {
		AxumStream(stream)
	}
}

impl Into<SplitStream<WebSocket>> for AxumStream {
	fn into(self) -> SplitStream<WebSocket> {
		self.0
	}
}

impl Stream for AxumStream {
	type Item = Result<Vec<u8>, axum::Error>;

	fn poll_next(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Option<Self::Item>> {
		match Pin::new(&mut self.0).poll_next(cx) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(None) => Poll::Ready(None),
			Poll::Ready(Some(res)) => match res {
				//Ok(item) => Poll::Ready(Some(Ok(item.into_bytes()))),
				//axum::extract::ws::Message::Binary(bin) => Ok(bin),
				Ok(axum::extract::ws::Message::Binary(bin)) => {
					Poll::Ready(Some(Ok(bin)))
				}
				Ok(_) => Poll::Ready(Some(Err(axum::Error::new(
					std::io::Error::new(
						std::io::ErrorKind::Other,
						"Non-binary message received",
					),
				)))),
				Err(e) => Poll::Ready(Some(Err(e))),
			},
		}
	}
}

//#[tokio::main]
#[tokio::test]
async fn axum_test() {
	/*
		let doc = Doc::new();
		// using a single static document shared among all the peers.
		let awareness: AwarenessRef = {
			{
				// pre-initialize code mirror document with some text
				let txt = doc.get_or_insert_text("codemirror");
				let mut txn = doc.transact_mut();
				txt.push(
					&mut txn,
					r#"function hello() {
	  console.log('hello world');
	}"#,
				);
			}
			Arc::new(RwLock::new(Awareness::new(doc)))
		};

		// open a broadcast group that listens to awareness and document updates
		// and has a pending message buffer of up to 32 updates
		let broadcast_group = Arc::new(BroadcastGroup::new(awareness.clone(), 32).await);
		*/
		
	let router = Router::new()
		.route("/", get(|| async { "Hello, World!" }))
		.route("/ws", get(handler_default))
		.route("/:roomName", get(handler_room_name))
		.with_state(AppState {
			docs: Arc::new(DashMap::new()),
		});

	let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

	println!("Listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, router.into_make_service())
		.await
		.unwrap();
	println!("Exiting!");
}

async fn handler_default(
	mut ws: WebSocketUpgrade,
	Path(room_name): Path<String>,
	State(state): State<AppState>,
) -> impl IntoResponse {
	//let room_name = room_name.unwrap_or_else(|| "default".to_string());
	//println!("Got request for room {}", room_name);
	
	/*let state = AppState {
		docs: Arc::new(DashMap::new()),
	};
	*/
	
	let room_name = String::from("default");
	ws.on_upgrade(|socket| {
		handle_socket(room_name, socket, state)
	})
}

async fn handler_room_name(
	mut ws: WebSocketUpgrade,
	Path(room_name): Path<String>,
	State(state): State<AppState>,
) -> impl IntoResponse {
	//let room_name = String::from("default");

	println!("Got request for room {}", room_name);
	ws.on_upgrade(|socket| handle_socket(room_name, socket, state))
}




async fn handle_socket(
	room_name: String,
	mut ws: WebSocket,
	state: AppState,
) {
	// if room does not exist, create it
	if !state.docs.contains_key(&room_name) {
		// create the awareness
		let awareness: AwarenessRef = {
			let doc = Doc::new();

			// pre-initialize code mirror document with some text
			let txt = doc.get_or_insert_text("codemirror");
			let mut txn = doc.transact_mut();
			txt.push(
				&mut txn,
				r#"function hello() {
  console.log('hello world');
}"#,
			);

			// create the awareness
			Arc::new(RwLock::new(Awareness::new(doc.clone())))
		};
		let broadcast = Arc::new(
			BroadcastGroup::new(awareness.clone(), 32).await,
		);
		state.docs.insert(
			room_name.clone(),
			YrsDoc {
				awareness,
				broadcast,
			},
		);
	}

	// get the document
	let yrs_doc_item = state.docs.get(&room_name).unwrap();
	let yrs_doc = yrs_doc_item.value();
	
	//let doc = yrs_doc.awareness.
	//let awareness = doc.awareness.clone();
	//let broadcast = yrs_doc.broadcast.clone();

	// create a sink and stream for the websocket
	let (mut sink, mut stream) = ws.split();
	//let sink = Arc::new(Mutex::new(sink));
	/*
	let sink = Arc::new(Mutex::new(WarpSink::from(sink)));

	// convert the stream into a binary stream
	let stream = stream.map(|message_result| {
		message_result.and_then(|message| match message {
			axum::extract::ws::Message::Binary(bin) => Ok(bin),
			_ => Err(axum::Error::new(std::io::Error::new(
				std::io::ErrorKind::Other,
				"Non-binary message received",
			))),
		})
	});
	*/
	let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = AxumStream::from(stream);
	
	let bsub = yrs_doc.broadcast.subscribe(sink, stream);
	
	

	match bsub.completed().await {
        Ok(_) => println!("broadcasting for channel finished successfully"),
        Err(e) => eprintln!("broadcasting for channel finished abruptly: {}", e),
    }
	
	/*
	// for each message received from the client
	while let Some(msg) = ws.recv().await {
		//
		let msg = if let Ok(msg) = msg {
			println!("Got message: {:?}", msg);
			msg
		//return;
		} else {
			// client disconnected
			println!("Client disconnected");
			return;
		};

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
			if ws
				.send(axum::extract::ws::Message::Binary(state_vector))
				.await
				.is_err()
			{
				// client disconnected
				println!("Client disconnected");
				return;
			}
		}
	}
	*/
}

