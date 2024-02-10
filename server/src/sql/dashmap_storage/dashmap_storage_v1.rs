#![deny(clippy::str_to_string)]

/*
use super::{
	alter_table,
	index,
	metadata,
	transaction
};
*/

use {
	async_trait::async_trait,
	//crossbeam_skiplist::SkipMap,
	dashmap::DashMap,
	futures::stream::{empty, iter},
	gluesql_core::{
		chrono::Utc,
		data::{CustomFunction as StructCustomFunction, Key, Schema, Value},
		error::Result,
		store::{
			CustomFunction, CustomFunctionMut, DataRow, RowIter, Store,
			StoreMut,
		},
	},
	serde::{Deserialize, Serialize, Serializer, Deserializer, de::{SeqAccess, Visitor}, ser::SerializeStruct, de},
	std::collections::{
		BTreeMap,
		//	HashMap
	},
	std::fmt,
	std::sync::Arc,
	//std::sync::RwLock as StdRwLock,
	tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Table {
	pub id_counter: i64,
	pub schema: Schema,
	pub rows: DashMap<Key, DataRow>,
}

impl Table {
	fn from_schema(schema: Schema) -> Self {
		Self {
			id_counter: 0,
			schema,
			rows: DashMap::new(),
		}
	}
	fn from_schema_and_rows(schema: Schema, rows: DashMap<Key, DataRow>) -> Self {
		Self {
			id_counter: 0,
			schema,
			rows,
		}
	}
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashmapStorage {
	pub tables: DashMap<String, Arc<Table>>,
	pub metadata: DashMap<String, DashMap<String, Value>>,
	pub functions: DashMap<String, StructCustomFunction>,
}

/*
#[async_trait(?Send)]
impl CustomFunction for DashmapStorage {
	async fn fetch_function(&self, func_name: &str) -> Result<Option<StructCustomFunction>> {
		match self.functions.get(&func_name.to_uppercase()) {
			Some(value_ref) => Ok(Some((*value_ref.value()).clone())),
			None => Ok(None),
		}
	}
	async fn fetch_all_functions(&self) -> Result<Vec<StructCustomFunction>> {
		Ok(self.functions.iter().map(|ref_multi| (*ref_multi.value()).clone()).collect())
	}
}

#[async_trait(?Send)]
impl CustomFunctionMut for DashmapStorage {
	async fn insert_function(&mut self, func: StructCustomFunction) -> Result<()> {
		self.functions.insert(func.func_name.to_uppercase(), func);
		Ok(())
	}

	async fn delete_function(&mut self, func_name: &str) -> Result<()> {
		self.functions.remove(&func_name.to_uppercase());
		Ok(())
	}
}
*/

#[async_trait(?Send)]
impl Store for DashmapStorage {
	async fn fetch_all_schemas(&self) -> Result<Vec<Schema>> {
		let mut schemas = self
			.tables
			.iter()
			.map(|table| table.schema.clone())
			.collect::<Vec<_>>();
		schemas.sort_by(|a, b| a.table_name.cmp(&b.table_name));

		Ok(schemas)
	}
	async fn fetch_schema(
		&self,
		table_name: &str,
	) -> Result<Option<Schema>> {
		self.tables
			.get(table_name)
			.map(|table| Ok(table.schema.clone()))
			.transpose()
	}

	async fn fetch_data(
		&self,
		table_name: &str,
		key: &Key,
	) -> Result<Option<DataRow>> {
		let row = self
			.tables
			.get(table_name)
			.and_then(|table| table.value()
			.rows
			.get(key)
			.map(|value_ref| (*value_ref).clone()));
			//.map(Clone::clone));
			// .value()
			// .clone()
		//);

		Ok(row)
	}

	async fn scan_data(
		&self,
		table_name: &str,
	) -> Result<RowIter> {
		let rows: RowIter = match self.tables.get(table_name) {
			Some(table) => {
				Box::pin(iter(table.rows.clone().into_iter().map(Ok)))
			}
			None => Box::pin(empty()),
		};

		Ok(rows)
	}
}

#[async_trait(?Send)]
impl StoreMut for DashmapStorage {
	async fn insert_schema(
		&mut self,
		schema: &Schema,
	) -> Result<()> {
		let created = DashMap::new();
		created.insert(
			"CREATED".to_owned(),
			Value::Timestamp(Utc::now().naive_utc()),
		);
		/*let created = HashMap::from([(
			"CREATED".to_owned(),
			Value::Timestamp(Utc::now().naive_utc()),
		)]);
		*/
		/*
		let meta = HashMap::from([(schema.table_name.clone(), created)]);
		self.metadata.extend(meta);
		*/
		let meta = DashMap::new();
		meta.insert(schema.table_name.clone(), created);
		self.metadata.extend(meta);

		let table_name = schema.table_name.clone();
		
		let table = Table::from_schema(schema.clone()); 
		
		self.tables.insert(table_name, Arc::new(table));

		Ok(())
	}

	async fn delete_schema(
		&mut self,
		table_name: &str,
	) -> Result<()> {
		self.tables.remove(table_name);
		self.metadata.remove(table_name);

		Ok(())
	}

	async fn append_data(
		&mut self,
		table_name: &str,
		rows: Vec<DataRow>,
	) -> Result<()> {
		if let Some(table) = self
		.tables
		.get_mut(table_name)
		.unwrap()
		.value()
		 {
			for row in rows {
				self.id_counter += 1;

				table.rows.insert(Key::I64(self.id_counter), row);
			}
		}

		Ok(())
	}

	async fn insert_data(
		&mut self,
		table_name: &str,
		rows: Vec<(Key, DataRow)>,
	) -> Result<()> {
		if let Some(table) = self.tables.get_mut(table_name) {
			for (key, row) in rows {
				table.rows.insert(key, row);
			}
		}

		Ok(())
	}

	async fn delete_data(
		&mut self,
		table_name: &str,
		keys: Vec<Key>,
	) -> Result<()> {
		if let Some(table) = self.tables.get_mut(table_name) {
			for key in keys {
				table.rows.remove(&key);
			}
		}

		Ok(())
	}
}
