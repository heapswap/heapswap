use gluesql::{
	core::store::{
		AlterTable, CustomFunction, CustomFunctionMut, Index, IndexMut,
		Metadata, Store, StoreMut,
	},
	prelude::*,
};
use std::sync::Arc;

trait FullStore:
	Store
	+ StoreMut
	+ AlterTable
	+ Index
	+ IndexMut
	+ Metadata
	+ CustomFunction
	+ CustomFunctionMut
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
			+ CustomFunctionMut,
	> FullStore for T
{
}

pub struct SubfieldStoreConfig {
	pub location: String,
}

#[derive(Clone)]
pub struct SubfieldStore {
	cache: Arc<dyn FullStore>,
	perma: Arc<dyn FullStore>,
}

impl SubfieldStore {
	pub async fn new(config: SubfieldStoreConfig) -> Result<Self, Error> {
		let cache: Arc<dyn FullStore> = Arc::new(SharedMemoryStorage::new());

		#[cfg(not(target_arch = "wasm32"))]
		let perma: Arc<dyn FullStore> =
			Arc::new(SledStorage::new(config.location.as_str())?);

		#[cfg(target_arch = "wasm32")]
		let perma: Arc<dyn FullStore> =
			Arc::new(IdbStorage::new(Some(config.location.to_string())).await?);

		Ok(SubfieldStore { cache, perma })
	}

	fn cache(&self) -> &Arc<dyn FullStore> {
		&self.cache
	}

	fn perma(&self) -> &Arc<dyn FullStore> {
		&self.perma
	}
}
