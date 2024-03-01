use {
	async_trait::async_trait,
	dashmap::DashMap,
	futures::{Future, stream},
	gluesql::core::{
		data::{CustomFunction as StructCustomFunction, Key, Schema, Value},
		error::Error as GlueError,
		store::{
			AlterTable, CustomFunction, CustomFunctionMut, DataRow, RowIter,
			Store, StoreMut,
		},
	},
	serde::{Deserialize, Serialize},
	std::pin::Pin,
	std::sync::{Arc, Mutex as StdMutex, RwLock as StdRwLock},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashTable {
	id_counter: Arc<StdMutex<i64>>,
	pub rows: DashMap<Key, DataRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashStorage {
	schemas: DashMap<String, Schema>,
	tables: DashMap<String, Arc<DashTable>>,
	functions: DashMap<String, StructCustomFunction>,
}

pub struct DashStorageShared {}

/*
// Functions
#[async_trait(?Send)]
impl CustomFunction for DashStorage {
    fn fetch_function<'life0, 'life1, 'async_trait>(
        &'life0 self,
        func_name: &'life1 str
    ) -> Pin<Box<dyn Future<Output = Result<Option<&'life0 CustomFunction>, GlueError>> + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            match self.functions.get(func_name) {
                Some(f) => Ok(Some(f.value() as &CustomFunction)),
                None => Err(GlueError::StorageMsg(format!("Function {} not found", func_name))),
            }
        })
    }

    fn fetch_all_functions<'life0, 'async_trait>(
        &'life0 self
    ) -> Pin<Box<dyn Future<Output = Result<Vec<&'life0 CustomFunction>, GlueError>> + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            Ok(self.functions.iter().map(|f| f.value() as &CustomFunction).collect())
        })
    }
}
*/

// Store
#[async_trait(?Send)]
impl Store for DashStorage {
	async fn fetch_schema(
		&self,
		table_name: &str,
	) -> Result<Option<Schema>, GlueError> {
		match self
			.schemas
			.get(table_name)
		{
			Some(s) => Ok(Some(
				s.value()
					.clone(),
			)),
			None => Err(GlueError::StorageMsg(format!(
				"Schema for table {} not found",
				table_name
			))),
		}
	}

	async fn fetch_all_schemas(&self) -> Result<Vec<Schema>, GlueError> {
		Ok(self
			.schemas
			.iter()
			.map(|s| {
				s.value()
					.clone()
			})
			.collect())
	}

	async fn fetch_data(
		&self,
		table_name: &str,
		key: &Key,
	) -> Result<Option<DataRow>, GlueError> {
		match self
			.tables
			.get(table_name)
		{
			Some(t) => Ok(t
				.value()
				.rows
				.get(key)
				.map(|r| {
					r.value()
						.clone()
				})),
			None => Err(GlueError::StorageMsg(format!(
				"Table {} not found",
				table_name
			))),
		}
	}

	async fn scan_data(
		&self,
		table_name: &str,
	) -> Result<RowIter, GlueError> {
		match self
			.tables
			.get(table_name)
		{
			Some(t) => {
				let iter = t
					.value()
					.rows
					.clone()
					.iter()
					.map(|r| {
						Ok((
							r.key()
								.clone(),
							r.value()
								.clone(),
						))
					});
				let stream = stream::iter(iter);
				Ok(Box::pin(stream))
			}
			None => Err(GlueError::StorageMsg(format!(
				"Table {} not found",
				table_name
			))),
		}
	}
}

#[async_trait(?Send)]
impl StoreMut for DashStorage {
	async fn insert_schema(
		&mut self,
		schema: &Schema,
	) -> Result<(), GlueError> {
		self.schemas
			.insert(
				schema
					.table_name
					.clone(),
				schema.clone(),
			);
		Ok(())
	}
	async fn delete_schema(
		&mut self,
		table_name: &str,
	) -> Result<(), GlueError> {
		self.schemas
			.remove(table_name);
		Ok(())
	}

	async fn append_data(
		&mut self,
		table_name: &str,
		rows: Vec<DataRow>,
	) -> Result<(), GlueError> {
		// create a vec of (Key, DataRow) from the DataRow by incrementing the id_counter on the table by the length of the rows

		// get the lock on the table id_counter
		let mut id_counter = self
			.tables
			.get(table_name)
			.ok_or(GlueError::StorageMsg(format!(
				"Table {} not found",
				table_name
			)))?
			.value()
			.id_counter
			.lock()
			.map_err(|_| {
				GlueError::StorageMsg(format!(
					"Table {} id_counter lock error",
					table_name
				))
			})?;

		// create a vec of keys starting from the id_counter and increasing by the length of the rows
		let keys = (*id_counter..*id_counter + rows.len() as i64)
			.collect::<Vec<_>>()
			.into_iter()
			.map(|id| Key::I64(id))
			.collect::<Vec<_>>();

		// zip the keys and the rows into a vec of (Key, DataRow)
		let rows = keys
			.into_iter()
			.zip(rows)
			.collect::<Vec<_>>();

		// call insert_data to insert the rows
		self.insert_data(table_name, rows)
			.await
	}

	async fn insert_data(
		&mut self,
		table_name: &str,
		rows: Vec<(Key, DataRow)>,
	) -> Result<(), GlueError> {
		self.tables
			.get(table_name)
			.ok_or(GlueError::StorageMsg(format!(
				"Table {} not found",
				table_name
			)))?
			.value()
			.clone()
			.rows
			.extend(
				rows.into_iter()
					.map(|(key, row)| {
						(
							key.clone(),
							row,
						)
					}),
			);
		Ok(())
	}
	
	async fn delete_data(
		&mut self,
		table_name: &str,
		keys: Vec<Key>,
	) -> Result<(), GlueError> {
		if let Some(table) = self.tables
			.get(table_name)
		{
			for key in keys {
				table
					.value()
					.clone()
					.rows
					.remove(&key);
			}
		} else {
			return Err(GlueError::StorageMsg(format!(
				"Table {} not found",
				table_name
			)));
		}
		Ok(())
	}
	
}
