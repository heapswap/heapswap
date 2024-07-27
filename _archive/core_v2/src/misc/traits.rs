use js_sys::Uint8Array;

// Able to be randomly generated
pub trait Randomable: Sized {
	fn random() -> Self;
}

// Able to be converted to and from bytes
pub trait Byteable<E>: Sized {
	fn to_bytes(&self) -> Uint8Array;
	fn from_bytes(bytes: &Uint8Array) -> Result<Self, E>;
}

// Able to be converted to and from a string
pub trait Stringable<E>: Sized {
	fn to_string(&self) -> String;
	fn from_string(string: &str) -> Result<Self, E>;
}
