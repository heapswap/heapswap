use crate::*;
use gluesql::{
	core::store::{
		AlterTable, CustomFunction, CustomFunctionMut, Index, IndexMut,
		Metadata, Store, StoreMut,
	},
	prelude::*,
};
use std::sync::Arc;
use thiserror::Error;

pub trait FullStore:
	Store
	+ StoreMut
	+ AlterTable
	+ Index
	+ IndexMut
	+ Metadata
	+ CustomFunction
	+ CustomFunctionMut
	+ Send
{
}

impl<
		T: Store
			+ StoreMut
			+ AlterTable
			+ Index
			+ IndexMut
			+ Metadata
			+ CustomFunction
			+ CustomFunctionMut
			+ Send,
	> FullStore for T
{
}

#[derive(Error, Debug)]
pub enum SubfieldStoreError {
	#[error("Failed to create SledStorage: {0}")]
	SledStorageError(String),
	#[error("Failed to create IdbStorage: {0}")]
	IdbStorageError(String),
}

type ThreadSafeFullStore = Arc<Mutex<Pin<Box<dyn FullStore + Send + 'static>>>>;
type ThreadSafeFullStoreGuard<'a> = MutexGuard<'a, Pin<Box<dyn FullStore + Send>>>;

#[derive(Clone)]
pub struct SubfieldStore {
	cache: ThreadSafeFullStore,
	perma: ThreadSafeFullStore,
}

impl SubfieldStore {
	pub async fn new(
		config: SubfieldStoreConfig,
	) -> Result<Self, SubfieldStoreError> {
		let cache: ThreadSafeFullStore =
			Arc::new(Mutex::new(Box::pin(gluesql::gluesql_memory_storage::MemoryStorage::default())));
			
		let store_path = config.store_path.clone();

		#[cfg(not(target_arch = "wasm32"))]
		let perma: ThreadSafeFullStore = match SledStorage::new(store_path.as_str()) {
			Ok(store) => Arc::new(Mutex::new(Box::pin(store))),
			Err(e) => {
				return Err(SubfieldStoreError::SledStorageError(
					e.to_string(),
				));
			}
		};

		#[cfg(target_arch = "wasm32")]
		let perma: ThreadSafeFullStore =
			match IdbStorage::new(Some(store_path.to_string())).await {
				Ok(store) => Arc::new(Mutex::new(Box::pin(store))),
				Err(e) => {
					return Err(SubfieldStoreError::IdbStorageError(
						e.to_string(),
					));
				}
			};

		Ok(SubfieldStore { cache, perma })
	}

	pub async fn cache<'a>(
		&'a self,
	) -> ThreadSafeFullStoreGuard<'a> {
		self.cache.lock().await
	}

	pub async fn perma<'a>(
		&'a self,
	) -> ThreadSafeFullStoreGuard<'a> {
		self.perma.lock().await
	}
}


