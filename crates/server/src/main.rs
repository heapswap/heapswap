#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]

use std::net::Ipv4Addr;

use heapswap_core::crypto::keys::{KeyPair, PublicKey};
use heapswap_core::ham_dht::{HamDHT, LocalNode, Node};
//use heapswap_protos::hello;
use futures_util::{SinkExt, StreamExt};
use heapswap_core::{bys, messages::*, traits::*, u256::*};
use once_cell::sync::Lazy;
use poem::{
	error::ResponseError,
	web::websocket::{Message, WebSocket},
	Result,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

//use heapswap_protos::hello::HelloWorldService;
//use std::fmt::{Formatter};

#[derive(Debug, strum::Display)]
pub enum FieldError {
	InvalidBase32,
	InvalidLength,

	#[strum(serialize = "Invalid Signer Base32")]
	InvalidSignerBase32,
	#[strum(serialize = "Invalid Signer Length")]
	InvalidSignerLength,

	#[strum(serialize = "Invalid Cosigner Base32")]
	InvalidCosignerBase32,
	#[strum(serialize = "Invalid Cosigner Length")]
	InvalidCosignerLength,

	#[strum(serialize = "Invalid Tangent Base32")]
	InvalidTangentBase32,
	#[strum(serialize = "Invalid Tangent Length")]
	InvalidTangentLength,
}

impl std::error::Error for FieldError {}

impl ResponseError for FieldError {
	fn status(&self) -> StatusCode {
		/*
		match self {
			FieldError::InvalidBase32 => StatusCode::BAD_REQUEST,
			FieldError::InvalidLength => StatusCode::BAD_REQUEST,
			// Map other variants to appropriate status codes
		}
		*/
		StatusCode::BAD_REQUEST
	}
}

use poem::{self, http::StatusCode, IntoResponse, Response as PoemResponse};
use poem::{
	get, handler,
	listener::TcpListener,
	web::{Json, Path},
	Route, Server,
};

enum FieldEnum {
	Signer,
	Cosigner,
	Tangent,
}

// Updated validate_field function with an additional parameter for field_enum
fn validate_field(field_enum: FieldEnum, field: &str) -> Result<Option<U256>> {
	if field == "_" {
		return Ok(None);
	}

	let field = bys::from_base32(&field).map_err(|_| match field_enum {
		FieldEnum::Signer => FieldError::InvalidSignerBase32,
		FieldEnum::Cosigner => FieldError::InvalidCosignerBase32,
		FieldEnum::Tangent => FieldError::InvalidTangentBase32,
	})?;

	if field.len() != 32 {
		return Err(match field_enum {
			FieldEnum::Signer => FieldError::InvalidSignerLength.into(),
			FieldEnum::Cosigner => FieldError::InvalidCosignerLength.into(),
			FieldEnum::Tangent => FieldError::InvalidTangentLength.into(),
		});
	}

	Ok(Some(U256::from_bytes(&field).unwrap()))
}

type GetHandlerError = FieldError;

#[derive(Debug, strum::Display)]
enum IndexError {
	#[strum(serialize = "Local Node Not Initialized")]
	LocalNodeNotInitialized,
}

impl ResponseError for IndexError {
	fn status(&self) -> StatusCode {
		StatusCode::BAD_REQUEST
	}
}

#[derive(Serialize, Deserialize)]
struct IndexResponse {
	node: Node,
	public_key: U256,
}

#[handler]
async fn index() -> Result<Json<IndexResponse>> {
	let local_node = DHT.read().await.local_node().clone();
	Ok(Json(IndexResponse {
		node: local_node.node().clone(),
		public_key: local_node.key_pair().public_key().as_u256().clone(),
	}))
}

#[handler]
fn main_get_handler(
	Path((signer, cosigner, tangent)): Path<(String, String, String)>,
) -> Result<Json<Field>> {
	let signer = validate_field(FieldEnum::Tangent, signer.as_str())?;
	let cosigner = validate_field(FieldEnum::Cosigner, cosigner.as_str())?;
	let tangent = validate_field(FieldEnum::Tangent, tangent.as_str())?;

	let field = Field::new(signer, cosigner, tangent);

	Ok(Json(field))
}

#[handler]
async fn main_ws_handler(ws: WebSocket) -> impl IntoResponse {
	ws.on_upgrade(|mut socket| async move {
		if let Some(Ok(Message::Text(text))) = socket.next().await {
			let _ = socket.send(Message::Text(text)).await;
		}
	})
}

static DHT: Lazy<RwLock<HamDHT>> = Lazy::new(|| {
	let dummy_node = Node {
		ipv4: Ipv4Addr::new(127, 0, 0, 1),
		ipv4_port: 1234,
		ipv6: None,
		ipv6_port: None,
	};

	let local_node = LocalNode::new(dummy_node, KeyPair::random().unwrap());
	RwLock::new(HamDHT::new(local_node, 32, 32))
});

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
	let app = Route::new()
		.at("/", get(index))
		.at("/:signer/:cosigner/:tangent", get(main_get_handler))
		.at("/ws", get(main_ws_handler));

	let port = std::env::var("PORT").unwrap_or("3000".to_string());
	let address = std::env::var("ADDRESS").unwrap_or("0.0.0.0".to_string());
	let listening_address = format!("{}:{}", address, port);
	let localhost_address = format!("http://localhost:{}", port);

	println!("Listening on {}", localhost_address);
	Server::new(TcpListener::bind(listening_address))
		.name("heapswap-server")
		.run(app)
		.await
}
