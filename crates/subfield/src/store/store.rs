use crate::*;
use gluesql::{
	core::store::{
		AlterTable, CustomFunction, CustomFunctionMut, Index, IndexMut,
		Metadata, Store, StoreMut,
	},
	prelude::*,
};
use std::sync::Arc;

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

// pub type ThreadsafeSubfieldStore = Arc<Mutex<SubfieldStore>>;

#[derive(Clone)]
pub struct SubfieldStore {
	cache: Arc<Mutex<dyn FullStore>>,
	perma: Arc<Mutex<dyn FullStore>>,
}

impl SubfieldStore {
	pub async fn new(config: swarm::SubfieldConfig) -> Result<Self, Error> {
		let cache: Arc<Mutex<dyn FullStore>> =
			Arc::new(Mutex::new(SharedMemoryStorage::new()));

		let store_path = config.store_path.clone();
		
		
		#[cfg(not(target_arch = "wasm32"))]
		let perma: Arc<Mutex<dyn FullStore>> = match SledStorage::new(store_path.as_str()) {
			Ok(store) => Arc::new(Mutex::new(store)),
			Err(e) => {
				println!("Failed to create SledStorage: {}", e);
				Arc::new(Mutex::new(SharedMemoryStorage::new()))
			}
		};

		#[cfg(target_arch = "wasm32")]
		let perma: Arc<Mutex<dyn FullStore>> = Arc::new(Mutex::new(
			IdbStorage::new(Some(store_path.to_string())).await?,
		));

		Ok(SubfieldStore { cache, perma })
	}

	pub async fn cache(&self) -> MutexGuard<dyn FullStore> {
		self.cache.lock().await
	}

	pub async fn perma(&self) -> MutexGuard<dyn FullStore> {
		self.perma.lock().await
	}
}

unsafe impl Send for SubfieldStore {}
