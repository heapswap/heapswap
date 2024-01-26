use anyhow::Result;
use arrow_array::array::{FixedSizeListArray, Float32Array, Int32Array};
use arrow_array::types::Float32Type;
use arrow_array::{RecordBatch, RecordBatchIterator, RecordBatchReader};
use arrow_schema::{DataType, Field, Schema};
use base64::read;
use dashmap::DashMap;
use futures_util::stream::TryStreamExt; // Add this line
use heapswap::embeddings::EmbeddingSession;
use heapswap::macros::*;
//use heapswap_macros::sled_zero_copy;
use std::path::PathBuf;
use std::sync::Arc;
use vectordb::connection::Database;
use vectordb::Connection;

#[test]
fn read_words() {
	use std::fs;
	use std::io::{self, Error};

	fn read_words_from_file(file_path: &str) -> io::Result<Vec<String>> {
		let content = fs::read_to_string(file_path)?;
		let words: Vec<String> = content
			.replace("\n", "\t")
			.split("\t")
			.map(|s| s.to_string())
			.collect();

		let words_only: Vec<String> = words
			.iter()
			.skip(1)
			.step_by(3)
			.map(|s| s.to_string())
			.collect();

		Ok(words_only)
	}

	let path = "datasets/english/words-10k.txt";
	let words = read_words_from_file(path).unwrap();

	//println!("words: {:?}", words);
}

#[test]
fn test_sled() {
	let db: sled::Db = sled::open("data/sled/db").unwrap();

	
	// insert and get
	let _ = db.insert(b"yo!", b"v1");
	assert_eq!(&db.get(b"yo!").unwrap().unwrap(), b"v1");

	// Atomic compare-and-swap.
	let _ = db.compare_and_swap(
		b"yo!",      // key
		Some(b"v1"), // old value, None for not present
		Some(b"v2"), // new value, None for delete
	)
	.unwrap();

	// Iterates over key-value pairs, starting at the given key.
	let scan_key: &[u8] = b"a non-present key before yo!";
	let mut iter = db.range(scan_key..);
	assert_eq!(&iter.next().unwrap().unwrap().0, b"yo!");
	assert_eq!(iter.next(), None);

	let _ = db.remove(b"yo!");
	assert_eq!(db.get(b"yo!"), Ok(None));

	let other_tree: sled::Tree = db.open_tree(b"cool db facts").unwrap();
	let _ = other_tree
		.insert(
			b"k1",
			&b"a Db acts like a Tree due to implementing Deref<Target = Tree>"
				[..],
		)
		.unwrap();
}
use {
byteorder::{BigEndian, LittleEndian},
zerocopy::{AsBytes, FromBytes, FromZeroes, Unaligned},
zerocopy::{
	byteorder::U64, U16, U32,
},
zerocopy_derive::*,
sled::{Db, IVec}
};

#[derive(FromBytes, FromZeroes, AsBytes, Unaligned)]
#[repr(C)]
struct Key {
    a: U64<BigEndian>,
    b: U64<BigEndian>,
}

#[sled_zero_copy]
struct Value {
    x: U64<LittleEndian>,
    y: U64<LittleEndian>,
}

#[test]
fn test_sled_structure() -> sled::Result<()> {
    let db: Db = sled::open("data/sled/structured")?;

    let key = Key { a: U64::new(1), b: U64::new(2) };
    let value = Value { x: U64::new(3), y: U64::new(4) };

    db.insert(key.as_bytes(), value.as_bytes())?;

    let retrieved_value = db.get(key.as_bytes())?;

    if let Some(retrieved_bytes) = retrieved_value {
        let (retrieved_value, _remaining) = zerocopy::Ref::<_, Value>::new_unaligned_from_prefix(retrieved_bytes.as_ref()).unwrap();
        println!("Retrieved value: x = {}, y = {}", retrieved_value.x.get(), retrieved_value.y.get());
    }

    Ok(())
}

//#[test]
fn test_vector_timing() -> Result<()> {
	let session = EmbeddingSession::new(
		"gte-small",
		"models/gte-small/model.onnx",
		"models/gte-small/tokenizer.json",
		512,
		2, //gte-small seems to have diminishing returns after 3 threads
	);

	let loop_count = 10;

	let sequence_map: DashMap<&str, &str> = DashMap::new();

	sequence_map.insert("short", "orangutans are cool");
	sequence_map.insert("medium", r#"Orangutans are great apes native to the rainforests of Indonesia and Malaysia. They are now found only in parts of Borneo and Sumatra, but during the Pleistocene they ranged throughout Southeast Asia and South China. Classified in the genus Pongo, orangutans were originally considered to be one species. From 1996, they were divided into two species: the Bornean orangutan (P. pygmaeus, with three subspecies) and the Sumatran orangutan (P. abelii). A third species, the Tapanuli orangutan (P. tapanuliensis), was identified definitively in 2017. The orangutans are the only surviving species of the subfamily Ponginae, which diverged genetically from the other hominids (gorillas, chimpanzees, and humans) between 19.3 and 15.7 million years ago. "#);
	sequence_map.insert("long", r#"Orangutans are great apes native to the rainforests of Indonesia and Malaysia. They are now found only in parts of Borneo and Sumatra, but during the Pleistocene they ranged throughout Southeast Asia and South China. Classified in the genus Pongo, orangutans were originally considered to be one species. From 1996, they were divided into two species: the Bornean orangutan (P. pygmaeus, with three subspecies) and the Sumatran orangutan (P. abelii). A third species, the Tapanuli orangutan (P. tapanuliensis), was identified definitively in 2017. The orangutans are the only surviving species of the subfamily Ponginae, which diverged genetically from the other hominids (gorillas, chimpanzees, and humans) between 19.3 and 15.7 million years ago.

	The most arboreal of the great apes, orangutans spend most of their time in trees. They have proportionally long arms and short legs, and have reddish-brown hair covering their bodies. Adult males weigh about 75 kg (165 lb), while females reach about 37 kg (82 lb). Dominant adult males develop distinctive cheek pads or flanges and make long calls that attract females and intimidate rivals; younger subordinate males do not and more resemble adult females. Orangutans are the most solitary of the great apes: social bonds occur primarily between mothers and their dependent offspring. Fruit is the most important component of an orangutan's diet; but they will also eat vegetation, bark, honey, insects and bird eggs. They can live over 30 years, both in the wild and in captivity.

	Orangutans are among the most intelligent primates. They use a variety of sophisticated tools and construct elaborate sleeping nests each night from branches and foliage. The apes' learning abilities have been studied extensively. There may be distinctive cultures within populations. Orangutans have been featured in literature and art since at least the 18th century, particularly in works that comment on human society. Field studies of the apes were pioneered by primatologist Birutė Galdikas and they have been kept in captive facilities around the world since at least the early 19th century.

	All three orangutan species are considered critically endangered. Human activities have caused severe declines in populations and ranges. Threats to wild orangutan populations include poaching (for bushmeat and retaliation for consuming crops), habitat destruction and deforestation (for palm oil cultivation and logging), and the illegal pet trade. Several conservation and rehabilitation organisations are dedicated to the survival of orangutans in the wild. "#);

	// timing
	for length in ["short", "medium", "long"].iter() {
		let mut embedding = vec![];

		let sequence = *sequence_map.get(length).unwrap().value();

		let sec = timeit_loops!(loop_count, {
			embedding = session.binary_quantize(session.embed(sequence)?)?;
		});

		println!(
			"{} sequence ({} tokens) : {} loops @ {} ms per loop",
			length,
			session.count_tokens(sequence).unwrap(),
			loop_count,
			(sec as f64 * 1000.0).round()
		);

		//println!("vector binary: {}", session.display_binary(embedding.clone())?);
		println!("vector hash: {}", session.display_base64(embedding)?);
	}

	Ok(())
}

//#[tokio::test]
pub async fn lance_test() {
	// connect to database
	let db = Database::connect("data/lance/db").await.unwrap();

	// define schema
	let schema = Arc::new(Schema::new(vec![
		Field::new("id", DataType::Int32, false),
		Field::new(
			"vector",
			DataType::FixedSizeList(
				Arc::new(Field::new("item", DataType::Float32, true)),
				128,
			),
			true,
		),
	]));

	// Create a RecordBatch stream.
	let batches = RecordBatchIterator::new(
		vec![RecordBatch::try_new(
			schema.clone(),
			vec![
				Arc::new(Int32Array::from_iter_values(0..10)),
				Arc::new(FixedSizeListArray::from_iter_primitive::<
					Float32Type,
					_,
					_,
				>((0..10).map(|_| Some(vec![Some(1.0); 128])), 128)),
			],
		)
		.unwrap()]
		.into_iter()
		.map(Ok),
		schema.clone(),
	);

	// Create a table.
	// Ensure that the type you're passing here implements RecordBatchReader
	match db
		.create_table(
			"my_table",
			Box::new(batches) as Box<dyn RecordBatchReader + Send>,
			None,
		)
		.await
	{
		Ok(_) => println!("Table created successfully"),
		Err(e) => println!("Error creating table: {}", e),
	}

	// get table
	let table = db.open_table("my_table").await.unwrap();

	// Create an index.
	table
		.create_index(&["vector"])
		.ivf_pq()
		.num_partitions(256)
		.build()
		.await
		.unwrap();

	// Search the index.
	let results = table
		.search(&[1.0; 128])
		.execute_stream()
		.await
		.unwrap()
		.try_collect::<Vec<_>>()
		.await
		.unwrap();
}
