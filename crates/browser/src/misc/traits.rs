use js_sys::Uint8Array;

// Able to be randomly generated
pub trait Randomable: Sized {
	fn random() -> Self;
}

// Able to be converted to and from bytes
pub trait Byteable<E>: Sized {
	fn toBytes(&self) -> Uint8Array;
	fn fromBytes(bytes: &Uint8Array) -> Result<Self, E>;
}

// Able to be converted to and from a string
pub trait Stringable<E>: Sized {
	fn toString(&self) -> String;
	fn fromString(string: &str) -> Result<Self, E>;
}
