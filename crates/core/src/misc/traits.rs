use bytes::Bytes;

// Able to be randomly generated
pub trait Randomable<E>: Sized {
	fn random() -> Result<Self, E>;
}

// Able to be converted to and from an array
pub trait Arrable<T, E>: Sized {
	fn to_arr(&self) -> T;
	fn from_arr(arr: &T) -> Result<Self, E>;
}

// Able to be converted to and from bytes
pub trait Byteable<E>: Sized {
	fn to_bytes(&self) -> Bytes;
	fn from_bytes(bytes: &Bytes) -> Result<Self, E>;
}

// Able to be converted to and from a string
pub trait Stringable<E>: Sized {
	fn to_string(&self) -> String;
	fn from_string(string: &str) -> Result<Self, E>;
}

// Able to be converted to and from a base32 string
pub trait Base32able<E>: Sized {
	fn to_base32(&self) -> String;
	fn from_base32(string: &str) -> Result<Self, E>;
}

// Able to be converted to and from a proto
pub trait Protoable<T, E>: Sized {
	fn to_proto(&self) -> T;
	fn from_proto(proto: &T) -> Result<Self, E>;
}
