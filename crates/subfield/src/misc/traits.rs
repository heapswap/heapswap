// Able to be randomly generated
pub trait Randomable: Sized {
	fn random() -> Self;
}

// Able to be converted to and from bytes
pub trait Byteable<E>: Sized {
	fn to_bytes(&self) -> Vec<u8>;
	fn from_bytes(bytes: &[u8]) -> Result<Self, E>;
}

// Able to be converted to and from a string
pub trait Stringable<E>: Sized {
	fn to_string(&self) -> String;
	fn from_string(string: &str) -> Result<Self, E>;
}

// Able to be converted to and from a lip2p keypair
pub trait Libp2pKeypairable<E>: Sized {
	fn to_libp2p_keypair(&self) -> libp2p::identity::Keypair;
	fn from_libp2p_keypair(
		libp2p_keypair: libp2p::identity::Keypair,
	) -> Result<Self, E>;
}
