use dashmap::mapref::one::Ref;

use crate::*;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelError {
	SendError(String),
	RecvError(String),
	HandleNotFound,
	HandleIsOneshotNotFound,
	ChannelClosed,
}

// pub trait ChannelHandle: Eq + std::hash::Hash + Clone {}

pub type ChannelHandle = u64;

pub struct ChannelManager<M> {
	current_handle: AtomicU64,
	handle_is_oneshot_tracker: DashMap<ChannelHandle, bool>,
	oneshot_tx: DashMap<ChannelHandle, OneshotSender<M>>,
	oneshot_rx: DashMap<ChannelHandle, OneshotReceiver<M>>,
	stream_tx: DashMap<ChannelHandle, UnboundedSender<M>>,
	stream_rx: DashMap<ChannelHandle, UnboundedReceiver<M>>,
}

// impl<M> ChannelManagerTrait<M> for ChannelManager<M> {
impl<M> ChannelManager<M> {
	pub fn new() -> Self {
		Self {
			current_handle: AtomicU64::new(0),
			handle_is_oneshot_tracker: DashMap::new(),
			oneshot_tx: DashMap::new(),
			oneshot_rx: DashMap::new(),
			stream_tx: DashMap::new(),
			stream_rx: DashMap::new(),
		}
	}

	pub fn handle(&self) -> ChannelHandle {
		self.current_handle.fetch_add(1, Ordering::SeqCst)
	}

	pub fn handle_is_oneshot(
		&self,
		handle: &ChannelHandle,
	) -> Result<bool, ChannelError> {
		match self.handle_is_oneshot_tracker.get(handle) {
			Some(res) => Ok(res.clone()),
			None => Err(ChannelError::HandleIsOneshotNotFound),
		}
	}

	/*
	 Oneshot
	*/
	pub fn create_oneshot(&self) -> ChannelHandle {
		let handle = self.handle();
		self.create_oneshot_with_handle(&handle);
		handle
	}

	pub fn create_oneshot_with_handle(&self, handle: &ChannelHandle) -> () {
		let (tx, rx) = oneshot::channel();
		self.oneshot_tx.insert(*handle, tx);
		self.oneshot_rx.insert(*handle, rx);
		self.handle_is_oneshot_tracker.insert(*handle, true);
	}

	pub fn send_oneshot(
		&self,
		handle: &ChannelHandle,
		val: M,
	) -> Result<(), ChannelError> {
		let (_handle, tx) = self
			.oneshot_tx
			.remove(&handle)
			.ok_or(ChannelError::HandleNotFound)?;

		tx.send(val).map_err(|_| {
			ChannelError::SendError("Failed to send oneshot message".into())
		})?;

		Ok(())
	}

	pub async fn recv_next_oneshot(
		&self,
		handle: &ChannelHandle,
	) -> Result<M, ChannelError> {
		let (_handle, mut rx) = self.oneshot_rx.remove(&handle).unwrap();
		let val = rx.await.unwrap();
		self.handle_is_oneshot_tracker.remove(&handle);
		Ok(val)
	}

	/*
	 Stream
	*/
	pub fn create_stream(&self) -> ChannelHandle {
		let handle = self.handle();
		self.create_stream_with_handle(&handle);
		handle
	}

	pub fn create_stream_with_handle(&self, handle: &ChannelHandle) -> () {
		let (tx, rx) = channel::<M>();
		self.stream_tx.insert(*handle, tx);
		self.stream_rx.insert(*handle, rx);
		self.handle_is_oneshot_tracker.insert(*handle, false);
	}

	pub fn delete_stream(&self, handle: &ChannelHandle) -> () {
		self.stream_tx.remove(&handle);
		self.stream_rx.remove(&handle);
		self.handle_is_oneshot_tracker.remove(&handle);
	}

	pub fn send_stream(
		&self,
		handle: &ChannelHandle,
		val: M,
	) -> Result<(), ChannelError> {
		if let Some(mut tx) = self.stream_tx.get_mut(&handle) {
			let _ = tx.send(val);
			Ok(())
		} else {
			Err(ChannelError::HandleNotFound)
		}
	}

	pub async fn recv_next_stream(
		&self,
		handle: &ChannelHandle,
	) -> Result<M, ChannelError> {
		if let Some(mut rx) = self.stream_rx.get_mut(&handle) {
			if let Some(val) = rx.next().await {
				Ok(val)
			} else {
				Err(ChannelError::RecvError("Stream closed".into()))
			}
		} else {
			Err(ChannelError::HandleNotFound)
		}
	}

	// pub async fn recv_stream(
	// 	&self,
	// 	handle: &ChannelHandle,
	// ) -> Result<UnboundedReceiver<M>, ChannelError> {
	// 	let (_handle, rx) = self.stream_rx.remove(&handle).unwrap();
	// 	Ok(rx)
	// }

	pub async fn recv_next_stream_or_oneshot(
		&self,
		handle: &ChannelHandle,
	) -> Result<M, ChannelError> {
		let is_oneshot = self.handle_is_oneshot(handle)?;
		if is_oneshot {
			self.recv_next_oneshot(handle).await
		} else {
			self.recv_next_stream(handle).await
		}
	}
}
