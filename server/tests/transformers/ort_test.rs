use anyhow::Result;
use base64::prelude::*;
use ndarray::Axis;
use ort::{
	tensor::OrtOwnedTensor, Environment, GraphOptimizationLevel, LoggingLevel,
	OrtError, Session, SessionBuilder, Value,
};
use std::sync::Arc;
use timeit::*;
use tokenizers::utils::truncation::*;
use tokenizers::{Encoding, Tokenizer};

pub struct EmbeddingSession {
	session: Session,
	tokenizer: Tokenizer,
	env: Arc<Environment>,
	threads: usize,
}

// this is a struct for an onnx runtime session for an embedding model
impl EmbeddingSession {
	// constructor
	pub fn new(
		session_name: &str,
		model_path: &str,
		tokenizer_path: &str,
		tokenizer_max_len: usize,
		threads: i16,
	) -> Self {
		// Initialize the ONNX Runtime environment
		let environment = Environment::builder()
			.with_name(session_name)
			.with_log_level(LoggingLevel::Error)
			.build()
			.unwrap()
			.into_arc();

		// Create a session builder
		let mut session_builder = SessionBuilder::new(&environment)
			.unwrap()
			.with_optimization_level(GraphOptimizationLevel::Level3)
			.unwrap();

		if threads > 0 {
			// use the number of threads specified
			session_builder =
				session_builder.with_intra_threads(threads).unwrap();
		}

		// Load the model to create the final session
		let session = session_builder.with_model_from_file(model_path).unwrap();

		// Initialize the tokenizer
		let mut tokenizer = Tokenizer::from_file(tokenizer_path).unwrap();
		let _ = tokenizer.with_truncation(Some(TruncationParams {
			max_length: tokenizer_max_len as usize, // max length is provided by the caller
			direction: TruncationDirection::Left,   // default value
			strategy: TruncationStrategy::LongestFirst, // default value
			stride: 0,                              // default value
		}));

		// Return the EmbeddingSession
		Self {
			session: session,
			tokenizer: tokenizer,
			env: environment,
			threads: threads as usize,
		}
	}

	// create an ndarray from a vector
	fn create_ndarray<F>(
		&self,
		tokenizer_output: &Encoding,
		func: F,
	) -> ndarray::Array2<i64>
	where
		F: Fn(&Encoding) -> &[u32],
	{
		ndarray::Array::from_shape_vec(
			(1, tokenizer_output.len()),
			func(tokenizer_output).iter().map(|&x| x as i64).collect(),
		)
		.unwrap()
	}
	
	fn create_cow_array<F>(
		&self,
		tokenizer_output: &Encoding,
		func: F,
	) -> ndarray::CowArray<'_, i64, ndarray::Dim<[usize; 2]>>
	where
		F: Fn(&Encoding) -> &[u32],
	{
		ndarray::CowArray::from(self.create_ndarray(tokenizer_output, func))
	}

	// embed a sequence of text
	fn embed(
		&self,
		sequence: &str,
	) -> Result<Vec<f32>> {
		
		// Step 1:	Tokenize the sequence
		let tokenizer_output = self.tokenizer.encode(sequence, true).unwrap();

		// debugging
		//println!("embedding {} tokens",  tokenizer_output.get_ids().len());

		// Step 2: run the session
		let outputs = self.session.run(vec![
			
			// input_ids
			Value::from_array(
				self.session.allocator(),
				&self.create_cow_array(&tokenizer_output, Encoding::get_ids).into_dyn(),
			) 
			.unwrap(),
			
			// attention_mask
			Value::from_array(
				self.session.allocator(),
				&self.create_cow_array(&tokenizer_output, Encoding::get_attention_mask).into_dyn(),
			)
			.unwrap(),
			
			// token_type_ids
			Value::from_array(
				self.session.allocator(),
				&self.create_cow_array(&tokenizer_output, Encoding::get_type_ids).into_dyn(),
			)
			.unwrap(),
			
		])?;
		
		// Step 3: Parse the outputs

		let result = outputs[0]
			.try_extract() // extract the sequence tensor
			.unwrap()
			.view()
			.mean_axis(Axis(1)) // average over the sequence
			.unwrap() 
			.to_owned()
			.as_slice()
			.unwrap()
			.to_vec(); // convert to Vec<f32>
			
		// Return the pooled output
		Ok(result)
	}

	// Quantize a vector of floats to binary and pack into i64
	fn binary_quantize(
		&self,
		unquantized_vector: Vec<f32>,
	) -> Result<Vec<i64>> {
		// Convert Vec<f32> to Vec<i64>
		let packed_vector: Vec<i64> = unquantized_vector
			.iter()
			.map(|&x| if x > 0.0 { 1 } else { 0 }) // quantize to binary
			.collect::<Vec<i8>>() // convert to Vec<i8>
			.chunks(64) // split into chunks of 64 bits
			.map(|chunk| {
				chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as i64))
			}) // pack into i64
			.collect(); // collect into Vec<i64>

		Ok(packed_vector)
	}

	// Display the binary representation of a vector
	fn display_binary(
		&self,
		vector: Vec<i64>,
	) -> Result<String> {
		// Convert Vec<i64> to Vec<String> and join it
		let result = vector
			.iter()
			.map(|&num| format!("{:064b}", num))
			.collect::<Vec<String>>()
			.join("");

		Ok(result)
	}

	// Display the base64 representation of a vector
	fn display_base64(
		&self,
		vector: Vec<i64>,
	) -> Result<String> {
		// Convert Vec<i64> to Vec<u8>
		let bytes: Vec<u8> = vector
			.iter()
			.flat_map(|&i| i.to_le_bytes().to_vec())
			.collect();

		// Encode to base64
		let result = BASE64_STANDARD.encode(&bytes);

		Ok(result)
	}
}

#[test]
fn main() -> Result<()> {
	let session = EmbeddingSession::new(
		"gte-small",
		"models/gte-small/model.onnx",
		"models/gte-small/tokenizer.json",
		512,
		0, // use default number of threads
	);

	let loop_count = 10;

	///*
	let sequence = r#"Orangutans are great apes native to the rainforests of Indonesia and Malaysia. They are now found only in parts of Borneo and Sumatra, but during the Pleistocene they ranged throughout Southeast Asia and South China. Classified in the genus Pongo, orangutans were originally considered to be one species. From 1996, they were divided into two species: the Bornean orangutan (P. pygmaeus, with three subspecies) and the Sumatran orangutan (P. abelii). A third species, the Tapanuli orangutan (P. tapanuliensis), was identified definitively in 2017. The orangutans are the only surviving species of the subfamily Ponginae, which diverged genetically from the other hominids (gorillas, chimpanzees, and humans) between 19.3 and 15.7 million years ago.

	The most arboreal of the great apes, orangutans spend most of their time in trees. They have proportionally long arms and short legs, and have reddish-brown hair covering their bodies. Adult males weigh about 75 kg (165 lb), while females reach about 37 kg (82 lb). Dominant adult males develop distinctive cheek pads or flanges and make long calls that attract females and intimidate rivals; younger subordinate males do not and more resemble adult females. Orangutans are the most solitary of the great apes: social bonds occur primarily between mothers and their dependent offspring. Fruit is the most important component of an orangutan's diet; but they will also eat vegetation, bark, honey, insects and bird eggs. They can live over 30 years, both in the wild and in captivity.

	Orangutans are among the most intelligent primates. They use a variety of sophisticated tools and construct elaborate sleeping nests each night from branches and foliage. The apes' learning abilities have been studied extensively. There may be distinctive cultures within populations. Orangutans have been featured in literature and art since at least the 18th century, particularly in works that comment on human society. Field studies of the apes were pioneered by primatologist Birutė Galdikas and they have been kept in captive facilities around the world since at least the early 19th century.

	All three orangutan species are considered critically endangered. Human activities have caused severe declines in populations and ranges. Threats to wild orangutan populations include poaching (for bushmeat and retaliation for consuming crops), habitat destruction and deforestation (for palm oil cultivation and logging), and the illegal pet trade. Several conservation and rehabilitation organisations are dedicated to the survival of orangutans in the wild. "#;
	//*/
	//let sequence = "orangutans are cool";

	//let sequence = r#"Orangutans are great apes native to the rainforests of Indonesia and Malaysia. They are now found only in parts of Borneo and Sumatra, but during the Pleistocene they ranged throughout Southeast Asia and South China. Classified in the genus Pongo, orangutans were originally considered to be one species. From 1996, they were divided into two species: the Bornean orangutan (P. pygmaeus, with three subspecies) and the Sumatran orangutan (P. abelii). A third species, the Tapanuli orangutan (P. tapanuliensis), was identified definitively in 2017. The orangutans are the only surviving species of the subfamily Ponginae, which diverged genetically from the other hominids (gorillas, chimpanzees, and humans) between 19.3 and 15.7 million years ago. "#;

	let mut embedding = vec![];
	let sec = timeit_loops!(loop_count, {
		embedding = session.binary_quantize(session.embed(sequence)?)?;
		//println!("{:?}", embedding);
	});
	println!(
		"{} loops at {} ms per loop",
		loop_count,
		(sec as f64 * 1000.0).round()
	);

	println!("{}", session.display_base64(embedding)?);

	Ok(())
}
