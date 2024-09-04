use crate::*;
use reqwest::{};
use reqwest_websocket::{RequestBuilderExt, Message, WebSocket};
use std::marker::PhantomData;

pub enum GamuxError {
	SendError(reqwest_websocket::Error),
	RecvError(reqwest_websocket::Error),
	SerializationError(bincode::Error),
	DeserializationError(bincode::Error),
}

/*
Gamux - Generational Arena Multiplexer
*/

struct Gamux<In: DeserializeOwned + Clone, Out: Serialize + Clone> {
	ws: Arc<Mutex<WebSocket>>,
	channel_arena: Arc<Mutex<Arena<Unichannel<In, Out>>>>,
	_marker: PhantomData<(In, Out)>,
}



impl<In, Out> Gamux<In, Out>
where
	In: DeserializeOwned + Clone,
	Out: Serialize + Clone,
{
	pub fn new(ws: WebSocket) -> Self {
		let ws = Arc::new(Mutex::new(ws));
		let channel_arena = Arc::new(Mutex::new(Arena::new()));	
		Self { ws, channel_arena, _marker: PhantomData }
	}
	
	pub async fn ws_lock(&self) -> MutexGuard<WebSocket> {
		self.ws.lock().await
	}
	
	pub async fn channel_arena_lock(&self) -> MutexGuard<Arena<Unichannel<In, Out>>> {
		self.channel_arena.lock().await
	}
	
	pub async fn open_channel(&self) -> (Unichannel<Out, In>, ArenaHandleBytes) {
		let (tx, rx) = channels::<In, Out>();
		let handle = self.channel_arena_lock().await.insert(rx);
		let handle_bytes = handle.to_bytes();
		(tx, handle_bytes)	
	}
	
	pub async fn send(&self, msg: Out) -> Result<(), GamuxError> {
		let serialized_msg = serialize(&msg).map_err(GamuxError::SerializationError)?;
		
		self.ws_lock().await.send(Message::Binary(serialized_msg)).await.map_err(|e| GamuxError::SendError(e))?;
		
		Ok(())
	}
	
	pub async fn poll(&self) -> Result<(), GamuxError> {
		let msg = self.ws_lock().await.next().await.unwrap();
		let deserialized_msg = deserialize::<In>(&msg).map_err(GamuxError::DeserializationError)?;
		Ok(())
	}



}
