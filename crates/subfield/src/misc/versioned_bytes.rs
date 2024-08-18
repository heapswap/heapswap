use crate::*;

pub type V96 = VersionedBytes<12>;
pub type V256 = VersionedBytes<32>;
pub type V512 = VersionedBytes<64>;

#[derive(Debug, strum::Display)]
pub enum VersionedBytesError {
	InvalidBase32,
	InvalidVersion,
}

#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct VersionedBytes<const DIM: usize> {
	#[getset(get = "pub")]
	version: u16,
	#[getset(get = "pub")]
	#[serde(with = "serde_bytes")]
	data: [u8; DIM],
	#[serde(skip)]
	string: OnceCell<String>,
}

impl<const DIM: usize> VersionedBytes<DIM> {
	
	pub fn new(version: u16, data: [u8; DIM]) -> Self {
		Self { version, data, string: OnceCell::new() }
	}
	
	pub fn leading_zeros(&self) -> u32 {
		let mut count = 0;
		for i in 0..DIM {
			if self.data[i] == 0 {
				count += 8;
			} else {
				count += self.data[i].leading_zeros();
				break;
			}
		}
		count
	}
}

/**
 * Stringable
*/
impl<const DIM: usize> Stringable<VersionedBytesError> for VersionedBytes<DIM> {
	fn to_string(&self) -> String {
		self.string.get_or_init(|| arr::to_base32((&self.to_vec()).as_ref())).clone()
	}

	fn from_string(s: &str) -> Result<Self, VersionedBytesError> {
		let vec = arr::from_base32(s).map_err(|_| VersionedBytesError::InvalidBase32)?;
		let arr: [u8; DIM] = vec.as_slice().try_into().map_err(|_| VersionedBytesError::InvalidVersion)?;
		VersionedBytes::from_arr(&arr)
	}
}

/**
 * Vecable
*/
impl<const DIM: usize> Vecable<VersionedBytesError> for VersionedBytes<DIM> {
	fn from_arr(arr: &[u8]) -> Result<Self, VersionedBytesError> {
		let (data, version) = arr.split_at(arr.len() - 2);
		let version = u16::from_le_bytes(version.try_into().unwrap());
		Ok(VersionedBytes::new(version, data.try_into().unwrap()))
	}

	fn to_vec(&self) -> Vec<u8> {
		let version_bytes: [u8; 2] = self.version.to_le_bytes();
		[self.data.as_slice(), &version_bytes].concat()
	}
}

/**
 * Byteable
*/
impl<const DIM: usize> Byteable<VersionedBytesError> for VersionedBytes<DIM> {
	fn to_bytes(&self) -> Bytes {
		Bytes::from(self.to_vec())
	}

	fn from_bytes(bytes: Bytes) -> Result<Self, VersionedBytesError> {
		Self::from_arr(&bytes.as_ref())
	}
}

/**
 * Randomable
*/
impl<const DIM: usize> Randomable for VersionedBytes<DIM> {
	fn random() -> Self {
		let data: [u8; DIM] = arr::random(DIM).try_into().unwrap();
		VersionedBytes::new(0, data)
	}
}

/**
 * Equality
*/
impl<const DIM: usize> PartialEq for VersionedBytes<DIM> {
	fn eq(&self, other: &Self) -> bool {
		self.data == other.data
	}
}

impl<const DIM: usize> Into<String> for VersionedBytes<DIM> {
	fn into(self) -> String {
		self.to_string()
	}
}

/**
 * Impls
*/
impl<const DIM: usize> From<String> for VersionedBytes<DIM> {
	fn from(string: String) -> Self {
		VersionedBytes::from_string(&string).unwrap()
	}
}

impl<const DIM: usize> From<&str> for VersionedBytes<DIM> {
	fn from(string: &str) -> Self {
		VersionedBytes::from_string(string).unwrap()
	}
}

/**
 * Clone
*/
impl<const DIM: usize> Clone for VersionedBytes<DIM> {
	fn clone(&self) -> Self {
		VersionedBytes::new(self.version, self.data.clone())
	}
}



#[test]
fn test_versioned_bytes() {
	let vec256 = V256::random();
	let vec256_serialized = serialize(&vec256).unwrap();
	// assert_eq!(vec256_serialized.len(), 32 + 2);
	assert_eq!(vec256, deserialize(&vec256_serialized).unwrap());
}
