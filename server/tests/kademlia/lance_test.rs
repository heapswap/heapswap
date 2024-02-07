use std::sync::Arc;

use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, Int32Array, RecordBatch, RecordBatchIterator};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;

use vectordb::Connection;
use vectordb::{connect, Result, Table, TableRef, };
use heapswap::macros::*;

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
    const TOTAL: usize = 1000;
    const DIM: usize = 32;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                DIM as i8,
            ),
            true,
        ),
    ]));

    // Create a RecordBatch stream.
    let batches = RecordBatchIterator::new(
        vec![RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int32Array::from_iter_values(0..TOTAL as i32)),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        (0..TOTAL).map(|_| Some(vec![Some(1.0); DIM])),
                        DIM as i32,
                    ),
                ),
            ],
        )
        .unwrap()]
        .into_iter()
        .map(Ok),
        schema.clone(),
    );
    
    // open the table
    let tbl = match db
        .create_table("my_table", Box::new(batches), None)
        .await {
        Ok(tbl) => tbl,
        Err(e) => {
            eprintln!("Error creating table: {}", e);
            return;
        }
    };

    let new_batches = RecordBatchIterator::new(
        vec![RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int32Array::from_iter_values(0..TOTAL as i32)),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        (0..TOTAL).map(|_| Some(vec![Some(1.0); DIM])),
                        DIM as i32,
                    ),
                ),
            ],
        )
        .unwrap()]
        .into_iter()
        .map(Ok),
        schema.clone(),
    );
    match tbl.add(Box::new(new_batches), None).await {
        Ok(_) => (),
        Err(e) => eprintln!("Error adding batches: {}", e),
    };

    match tbl
        .create_index(&["vector"])
        .ivf_pq()
        .num_partitions(8)
        .build()
        .await {
        Ok(_) => (),
        Err(e) => eprintln!("Error creating index: {}", e),
    };

    let batches = match tbl
        .search(&[1.0; 128])
        .limit(2)
        .execute_stream()
        .await {
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

    println!("{:?}", batches);

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
}