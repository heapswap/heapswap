use ndarray::{Array, Ix1};
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;
use crate::*;
use rand::{thread_rng, Rng};

#[derive(Debug, strum::Display)]
pub enum VectorError {
	UnableToSerialize,
	UnableToDeserialize,
	InvalidBase32,
	InvalidLength,
}


#[derive(Debug)]
pub struct Vector<const N: usize>
{
	// #[serde(skip)]
	data_i64: Array<i64, Ix1>,
	// #[serde(with = "serde_bytes")]
	data_u8: [u8; N],
	magnitude: f64,
	// #[serde(skip)]
	string: OnceCell<String>,
}

impl<const N: usize> Vector<N> {
	/**
	 * Constructor
	*/
	pub fn new(data_u8: [u8; N]) -> Vector<N> {
		Vector::<N>::from_u8(data_u8)
	}
	
	pub fn from_u8(data_u8: [u8; N]) -> Vector<N> {
		let data_i64: Vec<i64> = data_u8.iter().map(|&x| x as i64).collect();
		Vector { 
			data_i64: Array::from_vec(data_i64.clone()), 
			data_u8: data_u8, 
			magnitude: Self::calculate_magnitude(&data_i64),
			string: OnceCell::new()
		}
	}
	
	
	pub fn from_i8(data_i8: [i8; N]) -> Vector<N> {
		let data_u8: [u8; N] = data_i8.iter().map(|&x| x as u8).collect::<Vec<u8>>().try_into().unwrap();
		Vector::<N>::from_u8(data_u8)
	}
	
	
	pub fn random() -> Vector<N> {
		
		let data_u8: Array<u8, Ix1> = Array::random(N, Uniform::new(0, 255));
		
		Vector::<N>::from_u8(data_u8.to_vec().try_into().unwrap())
	}
	
	pub fn zero() -> Vector<N> {
		Vector::<N>::from_u8([0; N])
	}
	
	
	/**
	 * Getters
	*/
	pub fn magnitude(&self) -> f64 {
		self.magnitude
	}
	
	pub fn data_u8(&self) -> &[u8; N] {
		&self.data_u8
	}
	
	pub fn data_i64(&self) -> &Array<i64, Ix1> {
		&self.data_i64
	}

	/**
	 * Distance
	*/
	fn calculate_magnitude(data_i64: &[i64]) -> f64 {
		(data_i64.iter().map(|&x| x * x).sum::<i64>() as f64).sqrt()
	}

	fn calculate_dot_product(&self, other: &Vector<N>) -> i64 {
		// self.data_i64.iter().zip(other.data_i64.iter()).map(|(&a, &b)| a as i64 * b as i64).sum()
		self.data_i64.dot(&other.data_i64).into()
	}
	
	pub fn cosine_similarity(&self, other: &Vector<N>) -> f64 {
		self.calculate_dot_product(other) as f64 / (self.magnitude * other.magnitude)
	}
	
	pub fn hash(data: &[u8]) -> Vector<32> {
		let _hash: [u8; 32] = blake3::hash(data).into();
		Vector::<32>::from_u8(_hash)
	}

	pub fn verify(data: &[u8], data_hash: Vector<32>) -> bool {
		Vector::<32>::hash(data) == data_hash
	}
}

/**
 * Byteable
*/
impl<const N: usize> Byteable<VectorError> for Vector<N> {
	fn to_bytes(&self) -> Vec<u8> {
		self.data_u8.to_vec()
	}

	fn from_bytes(bytes: &[u8]) -> Result<Vector<N>, VectorError> {
		let bytes: [u8; N] =
			bytes.try_into().map_err(|_| VectorError::InvalidLength)?;
		Ok(Vector::<N>::from_u8(bytes))
	}
}

/**
 * Stringable
*/
impl<const N: usize> Stringable<VectorError> for Vector<N> {
	fn to_string(&self) -> String {
		self.string
			.get_or_init(|| arr::to_base32(&self.data_u8))
			.clone()
	}

	fn from_string(string: &str) -> Result<Self, VectorError> {
		let data_u8: [u8; N] = arr::from_base32(string).map(|a| a.try_into().unwrap()).map_err(|_| VectorError::InvalidBase32)?;
		Ok(Vector::<N>::from_u8(data_u8))
	}
}


/**
 * Equality
*/
impl<const N: usize> PartialEq for Vector<N> {
	fn eq(&self, other: &Self) -> bool {
		self.data_u8 == other.data_u8
	}
}

impl<const N: usize> Into<String> for Vector<N> {
	fn into(self) -> String {
		self.to_string()
	}
}

/**
 * Impls
*/
impl<const N: usize> From<String> for Vector<N> {
	fn from(string: String) -> Self {
		Vector::<N>::from_string(&string).unwrap()
	}
}

impl<const N: usize> From<&str> for Vector<N> {
	fn from(string: &str) -> Self {
		Vector::<N>::from_string(string).unwrap()
	}
}

/**
 * Clone
*/
impl<const N: usize> Clone for Vector<N> {
	fn clone(&self) -> Self {
		Vector::<N>::from_u8(self.data_u8.clone())
	}
}

/**
 * Serialization
*/

impl<const N: usize> Serialize for Vector<N> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let string_repr = self.to_string();
		serializer.serialize_str(&string_repr)
	}
}

impl<'de, const N: usize> Deserialize<'de> for Vector<N> {
	fn deserialize<D>(deserializer: D) -> Result<Vector<N>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let string_repr = String::deserialize(deserializer)
			.map_err(serde::de::Error::custom)?;
		Vector::<N>::from_string(&string_repr).map_err(serde::de::Error::custom)
	}
}


#[test]
fn test_distance(){
    
    let iterations = 100000;
    let population_size = 1000;
    
    let s = timeit::timeit_loops!(iterations, {
        let mut distances = Vec::<f64>::with_capacity(population_size);
    
        let key = Vector::<32>::random();
        
		/*
        for i in 0..population_size{
            distances.push(key.cosine_similarity(&Vector::<32>::random()));
        }  
        
        
        
        // sort and print the 10 smallest distances
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // println!("asc:  {:?}", &distances[0..3]);
        assert_ne!(distances[0], distances[1]);
        
        distances.sort_by(|a, b| b.partial_cmp(a).unwrap());
        // println!("desc: {:?}", &distances[0..3]);
        assert_ne!(distances[0], distances[1]);     
		*/   
        
    });
	println!("test_distance: {:.2}ms", s*iterations as f64);
}