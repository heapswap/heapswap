use crate::*;
use dashmap::DashMap;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot::{
	self, Receiver as OneshotReceiver, Sender as OneshotSender,
};
// use generational_arena::Arena;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone)]
pub enum PortalError {
	SendError(String),
	RecvError(String),
	HandleNotFound,
	PortalClosed,
}

// pub trait PortalHandle: Eq + std::hash::Hash + Clone {}

pub type PortalHandle = u64;

pub struct PortalManager<M> {
	current_handle: AtomicU64,
	oneshot_tx: DashMap<PortalHandle, OneshotSender<M>>,
	oneshot_rx: DashMap<PortalHandle, OneshotReceiver<M>>,
	stream_tx: DashMap<PortalHandle, UnboundedSender<M>>,
	stream_rx: DashMap<PortalHandle, UnboundedReceiver<M>>,
}

/*
pub trait PortalManagerTrait<M> {

	fn new() -> Self;
	fn handle(&self) -> PortalHandle;

	// oneshot
	fn create_oneshot(&self) -> PortalHandle;
	fn create_oneshot_with_handle(&self, handle: PortalHandle) -> ();
	fn send_oneshot(&self, handle: PortalHandle, val: M) -> Result<(), PortalError>;
	async fn recv_oneshot(&self, handle: PortalHandle) -> Result<M, PortalError>;

	// stream
	fn create_stream(&self) -> PortalHandle;
	fn create_stream_with_handle(&self, handle: PortalHandle) -> ();
	fn delete_stream(&self, handle: PortalHandle) -> ();
	fn send_stream(&self, handle: PortalHandle, val: M) -> Result<(), PortalError>;
	async fn recv_stream(&self, handle: PortalHandle) -> Result<M, PortalError>;
}
*/

// impl<M> PortalManagerTrait<M> for PortalManager<M> {
impl<M> PortalManager<M> {
	pub fn new() -> Self {
		Self {
			current_handle: AtomicU64::new(0),
			oneshot_tx: DashMap::new(),
			oneshot_rx: DashMap::new(),
			stream_tx: DashMap::new(),
			stream_rx: DashMap::new(),
		}
	}

	pub fn handle(&self) -> PortalHandle {
		self.current_handle.fetch_add(1, Ordering::SeqCst)
	}

	pub fn create_oneshot(&self) -> PortalHandle {
		let handle = self.handle();
		self.create_oneshot_with_handle(handle);
		handle
	}

	pub fn create_oneshot_with_handle(&self, handle: PortalHandle) -> () {
		let (tx, rx) = oneshot::channel();
		self.oneshot_tx.insert(handle.clone(), tx);
		self.oneshot_rx.insert(handle, rx);
	}

	pub fn send_oneshot(
		&self,
		handle: PortalHandle,
		val: M,
	) -> Result<(), PortalError> {
		let (_handle, mut tx) = self.oneshot_tx.remove(&handle).unwrap();

		let _ = tx.send(val);

		Ok(())
	}

	pub async fn recv_oneshot(
		&self,
		handle: PortalHandle,
	) -> Result<M, PortalError> {
		let (_handle, mut rx) = self.oneshot_rx.remove(&handle).unwrap();

		let val = rx.await.unwrap();
		Ok(val)
	}

	pub fn create_stream(&self) -> PortalHandle {
		let handle = self.handle();
		self.create_stream_with_handle(handle);
		handle
	}

	pub fn create_stream_with_handle(&self, handle: PortalHandle) -> () {
		let (tx, rx) = portal::<M>();
		self.stream_tx.insert(handle.clone(), tx);
		self.stream_rx.insert(handle, rx);
	}

	pub fn delete_stream(&self, handle: PortalHandle) -> () {
		self.stream_tx.remove(&handle);
		self.stream_rx.remove(&handle);
	}

	pub fn send_stream(
		&self,
		handle: PortalHandle,
		val: M,
	) -> Result<(), PortalError> {
		if let Some(mut tx) = self.stream_tx.get_mut(&handle) {
			let _ = tx.send(val);
			Ok(())
		} else {
			Err(PortalError::HandleNotFound)
		}
	}

	pub async fn recv_stream(
		&self,
		handle: PortalHandle,
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
}
