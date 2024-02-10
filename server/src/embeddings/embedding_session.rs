use anyhow::Result;
use base64::prelude::*;
use ndarray::Axis;
use num_cpus;
use ort::{
	Environment, GraphOptimizationLevel, InMemorySession, LoggingLevel,
	SessionBuilder, Value,
};
use tokenizers::utils::truncation::*;
use tokenizers::{Encoding, Tokenizer};

#[allow(dead_code)]
// holds a runtime for creating sentence embeddings
pub struct EmbeddingSession {
	session: InMemorySession<'static>,
	//model_bytes: Arc<[u8]>,
	tokenizer: Tokenizer,
	//env: Arc<Environment>,
	//threads: usize,
}

// for an onnx embedding model, specifically gte-small
impl EmbeddingSession {
	// constructor
	pub fn new(
		session_name: &str,
		model_bytes: &'static [u8],
		tokenizer_bytes: &[u8],
		tokenizer_max_len: usize,
		threads: i16, // 0 uses all available threads
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

		// make sure there are not more threads	than cpu cores
		let threads = if threads > 0 {
			std::cmp::min(threads, num_cpus::get() as i16)
		} else {
			num_cpus::get() as i16
		};

		if threads > 0 {
			// use the number of threads specified
			session_builder = session_builder
				.with_intra_threads(threads)
				.unwrap();
		}

		// Load the model to create the final session
		// for some reason, ort requires a static lifetime for the model bytes
		//let model_bytes_boxed: Box<[u8]> = model_bytes.to_vec().into_boxed_slice();
		//let model_bytes_static: &'static [u8] = Box::leak(model_bytes_boxed);

		//works for Arc<Vec<u8>>
		//let model_bytes_ptr = Arc::into_raw(model_bytes) as *const [u8];
		//let model_bytes_static: &'static [u8] = unsafe { &*model_bytes_ptr };

		let session = session_builder
			.with_model_from_memory(model_bytes)
			.unwrap();

		// Initialize the tokenizer
		let mut tokenizer = Tokenizer::from_bytes(tokenizer_bytes).unwrap();
		let _ = tokenizer.with_truncation(Some(TruncationParams {
			max_length: tokenizer_max_len as usize, // max length is provided by the caller
			direction: TruncationDirection::Left,   // default value
			strategy: TruncationStrategy::LongestFirst, // default value
			stride: 0,                              // default value
		}));

		// Return the EmbeddingSession
		Self {
			session: session,
			//model_bytes: model_bytes,
			tokenizer: tokenizer,
			//env: environment,
			//threads: threads as usize,
		}
	}

	// return the token count of a sequence
	pub fn count_tokens(
		&self,
		sequence: &str,
	) -> Result<usize> {
		// tokenize the sequence
		let tokenizer_output = self
			.tokenizer
			.encode(sequence, true)
			.unwrap();

		// return the number of tokens
		Ok(tokenizer_output
			.get_ids()
			.len())
	}

	// embed a sequence of text
	pub fn embed(
		&self,
		sequence: &str,
	) -> Result<Vec<f32>> {
		/*
		utility functions
		*/

		// create an ndarray from a vector
		fn create_ndarray<F>(
			tokenizer_output: &Encoding,
			func: F,
		) -> ndarray::Array2<i64>
		where
			F: Fn(&Encoding) -> &[u32],
		{
			ndarray::Array::from_shape_vec(
				(1, tokenizer_output.len()),
				func(tokenizer_output)
					.iter()
					.map(|&x| x as i64)
					.collect(),
			)
			.unwrap()
		}

		// create a cow array from a vector
		fn create_cow_array<F>(
			tokenizer_output: &Encoding,
			func: F,
		) -> ndarray::CowArray<'_, i64, ndarray::Dim<[usize; 2]>>
		where
			F: Fn(&Encoding) -> &[u32],
		{
			ndarray::CowArray::from(create_ndarray(tokenizer_output, func))
		}

		/*
		embedding functions
		*/

		// Step 1:	Tokenize the sequence
		let tokenizer_output = self
			.tokenizer
			.encode(sequence, true)
			.unwrap();

		// Step 2: run the session
		let outputs = self
			.session
			.run(vec![
				// input_ids
				Value::from_array(
					self.session
						.allocator(),
					&create_cow_array(&tokenizer_output, Encoding::get_ids)
						.into_dyn(),
				)
				.unwrap(),
				// attention_mask
				Value::from_array(
					self.session
						.allocator(),
					&create_cow_array(
						&tokenizer_output,
						Encoding::get_attention_mask,
					)
					.into_dyn(),
				)
				.unwrap(),
				// token_type_ids
				Value::from_array(
					self.session
						.allocator(),
					&create_cow_array(
						&tokenizer_output,
						Encoding::get_type_ids,
					)
					.into_dyn(),
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
	pub fn binary_quantize(
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
				chunk
					.iter()
					.fold(0, |acc, &bit| (acc << 1) | (bit as i64))
			}) // pack into i64
			.collect(); // collect into Vec<i64>

		Ok(packed_vector)
	}

	// Display the binary representation of a vector
	pub fn display_binary(
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
	pub fn display_base64(
		&self,
		vector: Vec<i64>,
	) -> Result<String> {
		// Convert Vec<i64> to Vec<u8>
		let bytes: Vec<u8> = vector
			.iter()
			.flat_map(|&i| {
				i.to_le_bytes()
					.to_vec()
			})
			.collect();

		// Encode to base64
		let result = BASE64_STANDARD.encode(&bytes);

		Ok(result)
	}
}
