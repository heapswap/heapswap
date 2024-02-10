use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

use arrow_array::types::Float32Type;
use arrow_array::{
	Array, FixedSizeListArray, Float32Array, Int32Array, RecordBatch,
	RecordBatchIterator,
};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use rand::Rng;

use heapswap::macros::*;
use vectordb::Connection;
use vectordb::{connect, Result, Table, TableRef};

type AddressVectorElementType = f32;
const ADDRESS_VECTOR_BITS: i32 = 512; // total bits in the address vector
const ADDRESS_VECTOR_ELEMENT_BITS: i32 =
	(std::mem::size_of::<AddressVectorElementType>() * 8) as i32; // bits in each element of the address vector
const ADDRESS_VECTOR_ELEMENT_COUNT: i32 =
	(ADDRESS_VECTOR_BITS) / ADDRESS_VECTOR_ELEMENT_BITS; // number of elements in the address vector
    
type hash64 = [f32; 2]; // 64-bit hash
type hash128 = [f32; 4]; // 128-bit hash
type hash256 = [f32; 8]; // 256-bit hash
type hash512 = [f32; 16]; // 512-bit hash
type hash768 = [f32; 24]; // 768-bit hash
type hash1024 = [f32; 32]; // 1024-bit hash

#[derive(Debug, Clone)]
struct KadAddress {
	vector: [AddressVectorElementType; ADDRESS_VECTOR_ELEMENT_COUNT as usize],
	ipv4: Option<Ipv4Addr>,
	ipv4_port: Option<u16>,
	ipv6: Option<Ipv6Addr>,
	ipv6_port: Option<u16>,
}

#[derive(Debug, Clone)]
struct KadNode {
	id: i32, // random, internally assigned
	address: KadAddress,
	ping: i32,
}

struct Embedding { // 384-bit hash
	vector: hash384,
}

struct UserAddress { // 256-bit hash
    e: hash256, //ed25519 public key
	u: hash128, // username hash
    //x: hash256, //x25519 public key
}

struct Point{ // 384-bit hash
	world: PointWorld,
	id: PointId,
	location: PointLocation,
}

struct PointWorld { // 128-bit hash
	world_hash: hash128,
}

struct PointId{ // 128-bit hash
    id_hash: hash128,
}

struct PointLocation { // 128-bit hash
	t: f32,
	x: f32,
	y: f32,
	z: f32,
}





impl PointLocation {
	fn as_array(&self) -> [f32; 4] {
		[self.t, self.x, self.y, self.z]
	}
}

#[tokio::test]
async fn main() {
	// connect to the database
	let uri = "../data/sample-lancedb";
	let db = match connect(uri).await {
		Ok(db) => db,
		Err(e) => {
			eprintln!("Error connecting to database: {}", e);
			return;
		}
	};

	// fetch the table names
	match db.table_names().await {
		Ok(names) => println!("Table names: {:?}", names),
		Err(e) => eprintln!("Error getting table names: {}", e),
	};

	// create a table
	const KAD_NODE_COUNT: i32 = 256;

	let kad_node_schema = Arc::new(Schema::new(vec![
		Field::new("id", DataType::Int32, false),
		Field::new(
			"address_vector",
			DataType::FixedSizeList(
				Arc::new(Field::new("item", DataType::Float32, true)),
				ADDRESS_VECTOR_ELEMENT_COUNT,
			),
			true,
		),
	]));

	// Generate random address vectors and create KadNodes
	let mut rng = rand::thread_rng();
	let kad_nodes: Vec<KadNode> = (0..KAD_NODE_COUNT)
		.map(|id| {
			let vector: [f32; ADDRESS_VECTOR_ELEMENT_COUNT as usize] =
				rng.gen();
			let address = KadAddress {
				vector,
				ipv4: Some(Ipv4Addr::new(127, 0, 0, 1)),
				ipv4_port: Some(8080),
				ipv6: None,
				ipv6_port: None,
			};
			KadNode {
				id,
				address,
				ping: 0,
			}
		})
		.collect();

	let self_node = kad_nodes[0].clone();

	// Create Int32Array for IDs
	let ids: Vec<i32> = kad_nodes.iter().map(|node| node.id).collect();
	let id_array =
		Arc::new(Int32Array::from_iter_values(ids)) as Arc<dyn Array>;

	// Create FixedSizeListArray for vectors
	let vectors: Vec<Vec<Option<f32>>> = kad_nodes
		.iter()
		.map(|node| node.address.vector.iter().map(|&v| Some(v)).collect())
		.collect();
	let vector_array =
		Arc::new(
			FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
				vectors.into_iter().map(Some),
				ADDRESS_VECTOR_ELEMENT_COUNT,
			),
		) as Arc<dyn Array>;

	// Create a RecordBatch stream.
	let kad_node_batches = RecordBatchIterator::new(
		vec![RecordBatch::try_new(
			kad_node_schema.clone(),
			vec![id_array, vector_array],
		)
		.unwrap()]
		.into_iter()
		.map(Ok),
		kad_node_schema.clone(),
	);

	// delete the table if it exists
	match db.drop_table("kad_node_table").await {
		Ok(_) => (),
		Err(e) => eprintln!("Error dropping table: {}", e),
	}

	// open the table
	let tbl = match db
		.create_table("kad_node_table", Box::new(kad_node_batches), None)
		.await
	{
		Ok(tbl) => tbl,
		Err(e) => {
			eprintln!("Error creating table: {}", e);
			return;
		}
	};

	/*
	let new_batches = RecordBatchIterator::new(
		vec![RecordBatch::try_new(
			kad_node_schema.clone(),
			vec![
				Arc::new(Int32Array::from_iter_values(0..KAD_NODE_COUNT)),
				Arc::new(FixedSizeListArray::from_iter_primitive::<
					Float32Type,
					_,
					_,
				>(
					(0..KAD_NODE_COUNT).map(|_| {
						Some(vec![
							Some(1.0);
							ADDRESS_VECTOR_ELEMENT_COUNT as usize
						])
					}),
					ADDRESS_VECTOR_ELEMENT_COUNT,
				)),
			],
		)
		.unwrap()]
		.into_iter()
		.map(Ok),
		kad_node_schema.clone(),
	);

	// add the batches to the table
	match tbl.add(Box::new(new_batches), None).await {
		Ok(_) => (),
		Err(e) => eprintln!("Error adding batches: {}", e),
	};
	*/

	match tbl
		.create_index(&["address_vector"])
		.ivf_pq()
		.num_partitions(8)
		.build()
		.await
	{
		Ok(_) => (),
		Err(e) => eprintln!("Error creating index: {}", e),
	};

	timeit!({
		let batches = match tbl
			.search(&self_node.address.vector)
			.limit(1)
			.execute_stream()
			.await
		{
			Ok(stream) => match stream.try_collect::<Vec<_>>().await {
				Ok(batches) => batches,
				Err(e) => {
					eprintln!("Error collecting batches: {}", e);
					return;
				}
			},
			Err(e) => {
				eprintln!("Error searching: {}", e);
				return;
			}
		};

		//println!("{:?}", batches);
	});

	/*

	let schema = Arc::new(Schema::new(vec![
		Field::new("id", DataType::Int32, false),
		Field::new("item", DataType::Utf8, true),
	]));
	let batches = RecordBatchIterator::new(vec![], schema.clone());
	match db.create_table("empty_table", Box::new(batches), None).await {
		Ok(_) => (),
		Err(e) => eprintln!("Error creating empty table: {}", e),
	};

	match tbl.delete("id > 24").await {
		Ok(_) => (),
		Err(e) => eprintln!("Error deleting: {}", e),
	};

	match db.drop_table("my_table").await {
		Ok(_) => (),
		Err(e) => eprintln!("Error dropping table: {}", e),
	};

	*/
}
