use dashmap::mapref::one::Ref;

use crate::*;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortalError {
	SendError(String),
	RecvError(String),
	HandleNotFound,
	HandleIsOneshotNotFound,
	PortalClosed,
}

// pub trait PortalHandle: Eq + std::hash::Hash + Clone {}

pub type PortalHandle = u64;

pub struct PortalManager<M> {
	current_handle: AtomicU64,
	handle_is_oneshot_tracker: DashMap<PortalHandle, bool>,
	oneshot_tx: DashMap<PortalHandle, OneshotSender<M>>,
	oneshot_rx: DashMap<PortalHandle, OneshotReceiver<M>>,
	stream_tx: DashMap<PortalHandle, UnboundedSender<M>>,
	stream_rx: DashMap<PortalHandle, UnboundedReceiver<M>>,
}

// impl<M> PortalManagerTrait<M> for PortalManager<M> {
impl<M> PortalManager<M> {
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

	pub fn handle(&self) -> PortalHandle {
		self.current_handle.fetch_add(1, Ordering::SeqCst)
	}

	pub fn handle_is_oneshot(
		&self,
		handle: &PortalHandle,
	) -> Result<bool, PortalError> {
		match self.handle_is_oneshot_tracker.get(handle) {
			Some(res) => Ok(res.clone()),
			None => Err(PortalError::HandleIsOneshotNotFound),
		}
	}

	/*
	 Oneshot
	*/
	pub fn create_oneshot(&self) -> PortalHandle {
		let handle = self.handle();
		self.create_oneshot_with_handle(&handle);
		handle
	}

	pub fn create_oneshot_with_handle(&self, handle: &PortalHandle) -> () {
		let (tx, rx) = oneshot::channel();
		self.oneshot_tx.insert(*handle, tx);
		self.oneshot_rx.insert(*handle, rx);
		self.handle_is_oneshot_tracker.insert(*handle, true);
	}

	pub fn send_oneshot(
		&self,
		handle: &PortalHandle,
		val: M,
	) -> Result<(), PortalError> {
		let (_handle, tx) = self
			.oneshot_tx
			.remove(&handle)
			.ok_or(PortalError::HandleNotFound)?;

		tx.send(val).map_err(|_| {
			PortalError::SendError("Failed to send oneshot message".into())
		})?;

		Ok(())
	}

	pub async fn recv_next_oneshot(
		&self,
		handle: &PortalHandle,
	) -> Result<M, PortalError> {
		let (_handle, mut rx) = self.oneshot_rx.remove(&handle).unwrap();
		let val = rx.await.unwrap();
		self.handle_is_oneshot_tracker.remove(&handle);
		Ok(val)
	}

	/*
	 Stream
	*/
	pub fn create_stream(&self) -> PortalHandle {
		let handle = self.handle();
		self.create_stream_with_handle(&handle);
		handle
	}

	pub fn create_stream_with_handle(&self, handle: &PortalHandle) -> () {
		let (tx, rx) = portal::<M>();
		self.stream_tx.insert(*handle, tx);
		self.stream_rx.insert(*handle, rx);
		self.handle_is_oneshot_tracker.insert(*handle, false);
	}

	pub fn delete_stream(&self, handle: &PortalHandle) -> () {
		self.stream_tx.remove(&handle);
		self.stream_rx.remove(&handle);
		self.handle_is_oneshot_tracker.remove(&handle);
	}

	pub fn send_stream(
		&self,
		handle: &PortalHandle,
		val: M,
	) -> Result<(), PortalError> {
		if let Some(mut tx) = self.stream_tx.get_mut(&handle) {
			let _ = tx.send(val);
			Ok(())
		} else {
			Err(PortalError::HandleNotFound)
		}
	}

	pub async fn recv_next_stream(
		&self,
		handle: &PortalHandle,
	) -> Result<M, PortalError> {
		if let Some(mut rx) = self.stream_rx.get_mut(&handle) {
			if let Some(val) = rx.next().await {
				Ok(val)
			} else {
				Err(PortalError::RecvError("Stream closed".into()))
			}
		} else {
			Err(PortalError::HandleNotFound)
		}
	}

	// pub async fn recv_stream(
	// 	&self,
	// 	handle: &PortalHandle,
	// ) -> Result<UnboundedReceiver<M>, PortalError> {
	// 	let (_handle, rx) = self.stream_rx.remove(&handle).unwrap();
	// 	Ok(rx)
	// }

	pub async fn recv_next_stream_or_oneshot(
		&self,
		handle: &PortalHandle,
	) -> Result<M, PortalError> {
		let is_oneshot = self.handle_is_oneshot(handle)?;
		if is_oneshot {
			self.recv_next_oneshot(handle).await
		} else {
			self.recv_next_stream(handle).await
		}
	}
}
