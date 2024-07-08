use crate::crypto::snow::*;
use flume::{Receiver, Sender};
use futures::Stream;
use futures_util::{Sink, SinkExt, TryStreamExt};
use getset::{Getters, MutGetters, Setters};
use reqwest::Client;
use reqwest_websocket::{Message, RequestBuilderExt, WebSocket};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::marker::PhantomData;

#[derive(Debug)]
pub enum NoiseWebsocketError {
	WebsocketUpgrade(reqwest_websocket::Error),
	IntoWebsocket(reqwest_websocket::Error),
	WebsocketSend(reqwest_websocket::Error),
	WebsocketReceive(reqwest_websocket::Error),
	NoiseHandshakeFailed,
	RecvFailed,
	SerializeFailed(bincode::Error),
	DeserializeFailed(bincode::Error),
	EncryptFailed,
	DecryptFailed,
}

//pub trait WebSocketStream:
//	Stream<Item = Result<Message, reqwest_websocket::Error>>
//	+ Sink<Message, Error = reqwest_websocket::Error>
//	+ Unpin
//	+ Send
//{
//}

//impl<T> WebSocketStream for T where
//	T: Stream<Item = Result<Message, reqwest_websocket::Error>>
//		+ Sink<Message, Error = reqwest_websocket::Error>
//		+ Unpin
//		+ Send
//{
//}

pub trait WebSocketStream = dyn impl Stream

type WsStream = Box<dyn WebSocketStream>;

#[derive(Getters, Setters, MutGetters)]
pub struct NoiseWebsocket<M> {
	_m: PhantomData<M>,
	websocket: WsStream,
	noise_buffer: SnowBuffer,
	noise_transport: TransportState,
}
pub trait Serializable: Serialize + for<'de> Deserialize<'de> {}
impl<T> Serializable for T where T: Serialize + for<'de> Deserialize<'de> {}

async fn read_empty(
	websocket: &mut WsStream,
	noise_handshake: &mut HandshakeState,
	noise_buffer: &mut SnowBuffer,
) -> Result<(), NoiseWebsocketError> {
	match websocket.try_next().await {
		Ok(Some(Message::Binary(data))) => {
			noise_handshake
				.read_message(&data, noise_buffer)
				.map_err(|_| NoiseWebsocketError::NoiseHandshakeFailed)?;
			Ok(())
		}
		Err(e) => Err(NoiseWebsocketError::WebsocketReceive(e)),
		_ => Err(NoiseWebsocketError::NoiseHandshakeFailed),
	}
}

async fn write_empty(
	websocket: &mut WsStream,
	noise_handshake: &mut HandshakeState,
	noise_buffer: &mut SnowBuffer,
) -> Result<(), NoiseWebsocketError> {
	let len = noise_handshake.write_message(&[], noise_buffer).unwrap();
	let _ = websocket
		//.send(Message::Binary(noise_buffer[..len].to_vec()))
		.send(noise_buffer[..len].to_vec().into())
		.await
		.map_err(|e| NoiseWebsocketError::WebsocketSend(e))?;
	Ok(())
}

impl<M> NoiseWebsocket<M>
where
	M: Serializable + Send + Sync + 'static,
	//M: Serializable,
{
	async fn connect(url: &str) -> Result<WebSocket, NoiseWebsocketError> {
		// Connect to the websocket
		let websocket = Client::default()
			.get(url)
			.upgrade() // Prepares the WebSocket upgrade.
			.send()
			.await
			.map_err(|e| NoiseWebsocketError::WebsocketUpgrade(e))?
			.into_websocket()
			.await
			.map_err(|e| NoiseWebsocketError::IntoWebsocket(e))?;

		Ok(websocket)
	}

	pub async fn initiator(
		url: &str,
	) -> Result<NoiseWebsocket<M>, NoiseWebsocketError> {
		let websocket: Box<WebSocket> = Box::new(Self::connect(url).await?);
		let mut websocket: Box<dyn WebSocketStream> = Box::new(*websocket);

		let mut noise_buffer = snow_buffer_default();
		let mut noise_handshake = snow_initiator(&snow_keypair());

		// noise handshake

		// -> e
		write_empty(&mut websocket, &mut noise_handshake, &mut noise_buffer)
			.await?;
		// <- e, ee, s, es
		read_empty(&mut websocket, &mut noise_handshake, &mut noise_buffer)
			.await?;
		// -> s, se
		write_empty(&mut websocket, &mut noise_handshake, &mut noise_buffer)
			.await?;
		// convert to transport mode
		let noise_transport = noise_handshake.into_transport_mode().unwrap();

		Ok(NoiseWebsocket {
			_m: PhantomData,
			websocket,
			noise_buffer,
			noise_transport,
		})
	}

	pub async fn responder(
		mut websocket: WsStream,
	) -> Result<NoiseWebsocket<M>, NoiseWebsocketError> {
		let mut noise_buffer = snow_buffer_default();
		let mut noise_handshake = snow_initiator(&snow_keypair());

		// noise handshake

		// <- e
		read_empty(&mut websocket, &mut noise_handshake, &mut noise_buffer)
			.await?;
		// -> e, ee, s, es
		write_empty(&mut websocket, &mut noise_handshake, &mut noise_buffer)
			.await?;
		// <- s, se
		read_empty(&mut websocket, &mut noise_handshake, &mut noise_buffer)
			.await?;
		// convert to transport mode
		let noise_transport = noise_handshake.into_transport_mode().unwrap();

		Ok(NoiseWebsocket {
			_m: PhantomData,
			websocket,
			noise_buffer,
			noise_transport,
		})
	}

	pub async fn send(
		&mut self,
		message: M,
	) -> Result<(), NoiseWebsocketError> {
		let message = self.encrypt(message).await?;
		self.websocket
			//.send(Message::Binary(message))
			.send(message.into())
			.await
			.map_err(|e| NoiseWebsocketError::WebsocketSend(e))?;
		Ok(())
	}
	
	pub async fn encrypt(&mut self, message: M) -> Result<Vec<u8>, NoiseWebsocketError> {
		let message = bincode::serialize(&message).map_err(|e| NoiseWebsocketError::SerializeFailed(e))?;
		let len = self.noise_transport.write_message(&message, &mut self.noise_buffer).map_err(|_| NoiseWebsocketError::EncryptFailed)?;
		return Ok(self.noise_buffer[..len].to_vec());
	}

	//pub async fn recv(&mut self) -> Result<M, NoiseWebsocketError> {
	pub async fn recv(&mut self) -> Message {
		return self.websocket.try_next().await.unwrap().unwrap();
		//let Some(message) = self.websocket.try_next().await.unwrap();
		//return message 
		// {
		//	match message {
		//		Message::Binary(data) => {
		//			self.decrypt(data).await?;
		//		}, 
		//		_ => {}
		//	}
		//}
		//Err(NoiseWebsocketError::RecvFailed)
	}
	
	pub async fn decrypt(&mut self, message: Vec<u8>) -> Result<M, NoiseWebsocketError> {
		let len = self
			.noise_transport
			.read_message(&message, &mut self.noise_buffer)
			.map_err(|_| NoiseWebsocketError::DecryptFailed)?;
		let message: M =
			bincode::deserialize(&self.noise_buffer[..len]).map_err(|e| NoiseWebsocketError::DeserializeFailed(e))?;
		return Ok(message);	
	}
	
	
}
