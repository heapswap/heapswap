#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]
use futures_util::{SinkExt, StreamExt};
use poem::{
    get, handler, Server,
	listener::TcpListener,
    web::websocket::{Message, WebSocket},
    IntoResponse, Route, Result, error
};
use serde::{Deserialize, Serialize};
use heapswap_core::{crypto::snow::*, arr::*, traits::*};
use lazy_static::lazy_static;
use snow::{Builder, Keypair, params::NoiseParams};
use std::sync::Mutex;
use bincode::{serialize, deserialize};
use eyre::eyre;

#[derive(Serialize, Deserialize)]
struct MessageData{
	message: String,
}

//const SNOW_BUILDER: Builder = Builder::new(SNOW_PARAMS.clone());
	//static ref SNOW_KEYPAIR: Keypair = SNOW_BUILDER.generate_keypair().unwrap();
	
lazy_static! {
	static ref SNOW_BUILDER: Mutex<Builder<'static>> = Mutex::new(Builder::new(SNOW_PARAMS.clone()));
	static ref SNOW_KEYPAIR: Keypair = SNOW_BUILDER.lock().unwrap().generate_keypair().unwrap();
}

#[handler]
async fn main_ws_handler(ws: WebSocket) -> impl IntoResponse {
    ws.on_upgrade(|mut socket|  async move  {
		
		// snow handshake
		let mut snow_buf = vec![0u8; 65535];
		
		let snow_builder = Builder::new(SNOW_PARAMS.clone());
		let mut snow_responder = snow_builder
			.local_private_key(&SNOW_KEYPAIR.private)
			//.unwrap()
			.psk(3, SNOW_SECRET)
			//.unwrap()
			.build_responder()
			.map_err(error::InternalServerError).unwrap();
	
		// Handshake part 1	
		match socket.next().await {
			Some(Ok(Message::Binary(data))) => {
				let _ = snow_responder.read_message(&data, &mut snow_buf).unwrap();
				let len = snow_responder.write_message(&[], &mut snow_buf).unwrap();
				let _ = socket.send(Message::Binary(snow_buf[..len].to_vec())).await;	
			},
			_ => {
				return error::InternalServerError(eyre!("Handshake part 1 failed").into());
			},
		}
		
		// Handshake part 2
		match socket.next().await {
			Some(Ok(Message::Binary(data))) => {
				let _ = snow_responder.read_message(&data, &mut snow_buf).unwrap();
			},
			_ => {
			},
		}
		
		// Handshake complete, transition to transport mode
		let mut snow_responder = snow_responder.into_transport_mode().unwrap();
		
		
		
        while let Some(message) = socket.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    // Handle text message
                    let _ = socket.send(Message::Text(text)).await;
                },
                Ok(Message::Binary(encrypted_data)) => {
                    // Handle binary message
                    // For example, echo the binary data back
                    //let _ = socket.send(Message::Binary(data)).await;
					
					let len = snow_responder.read_message(&encrypted_data, &mut snow_buf).map_err(error::InternalServerError).unwrap();
					let decrypted_message: MessageData = deserialize(&snow_buf[..len]).map_err(error::InternalServerError).unwrap();
					
					println!("Received message: {}", decrypted_message.message);
					
				
                },
                //Err(e) => {
                //    // Handle any errors
                //    //eprintln!("Error handling websocket message: {:?}", e);
				//	return error::InternalServerError(eyre!("Loop 1 failed").into());
					
                //},
				/*
                Ok(Message::Ping(data)) => {
                    // Handle ping message
                    // Usually, respond with a pong containing the same payload
                    let _ = socket.send(Message::Pong(data)).await;
                },
                Ok(Message::Pong(_)) => {
                    // Handle pong message
                    // Pong messages are usually sent in response to pings; no action required
                },
                Ok(Message::Close(_)) => {
                    // Handle close message
                    // You might want to close the socket connection here
                    break;
                },
				*/
				_ => {},
            }
			
        }
		
		return error::InternalServerError(eyre!("Loop Exited").into());
		
    })
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
	let app = Route::new()
		//.at("/", get(index))
		//.at("/:signer/:cosigner/:tangent", get(main_get_handler))
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