use crate::*;
use subfield_proto::versioned_bytes::VersionedBytes;

pub enum VersionedBytesError {
	InvalidVersion,
	InvalidBase32,
	InvalidProto,
}

impl Stringable<VersionedBytesError> for VersionedBytes {
	fn to_string(&self) -> String {
		format!("{}:v{}", arr::to_base32(self.data.as_ref()), self.version)
	}

	fn from_string(s: &str) -> Result<Self, VersionedBytesError> {
		let (data, version) = s
			.split_once(":v")
			.ok_or(VersionedBytesError::InvalidBase32)?;
		let version = version
			.parse::<u32>()
			.map_err(|_| VersionedBytesError::InvalidVersion)?;
		let data = arr::from_base32(data)
			.map_err(|_| VersionedBytesError::InvalidBase32)?;
		Ok(VersionedBytes { version, data })
	}
}

impl U256able for VersionedBytes {
	fn u256(&self) -> &[u8; 32] {
		match <Vec<u8> as AsRef<[u8]>>::as_ref(&self.data).try_into() {
			Ok(array) => array,
			Err(_) => panic!("Invalid length of data"),
		}
	}
}

impl RandomLengthable for VersionedBytes {
	fn random_length(length: usize) -> Self {
		VersionedBytes {
			version: 0,
			data: arr::random(length),
		}
	}
}

impl Randomable for VersionedBytes {
	fn random() -> Self {
		VersionedBytes {
			version: 0,
			data: arr::random(32),
		}
	}
}

impl Byteable<VersionedBytesError> for VersionedBytes {
	fn to_bytes(&self) -> Bytes {
		Bytes::from(arr::from_proto::<VersionedBytes>(&self))
	}

	fn from_bytes(bytes: Bytes) -> Result<Self, VersionedBytesError> {
		VersionedBytes::decode(bytes.as_ref())
			.map_err(|_| VersionedBytesError::InvalidProto)
	}
}
