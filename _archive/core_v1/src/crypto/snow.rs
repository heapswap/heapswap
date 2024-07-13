use lazy_static::lazy_static;
pub use snow::{
	params::NoiseParams, Builder, HandshakeState, Keypair, TransportState,
};
use std::sync::Mutex;
use std::{
	io::{self, Read, Write},
	net::{TcpListener, TcpStream},
};

pub type SnowBuffer = Vec<u8>;

lazy_static! {
	pub static ref SNOW_PARAMS: NoiseParams =
		"Noise_NN_25519_ChaChaPoly_BLAKE2s".parse().unwrap();
}

// generate a new keypair
pub fn snow_keypair() -> Keypair {
	Builder::new(SNOW_PARAMS.clone())
		.generate_keypair()
		.unwrap()
}

// generate a new buffer
pub fn snow_buffer(len: usize) -> SnowBuffer {
	vec![0u8; len]
}

pub fn snow_buffer_default() -> SnowBuffer {
	snow_buffer(65535)
}

// create a new snow handshake builder
pub fn snow_builder(keypair: &Keypair) -> Builder {
	Builder::new(SNOW_PARAMS.clone()).local_private_key(&keypair.private)
}

pub fn snow_responder(keypair: &Keypair) -> HandshakeState {
	snow_builder(keypair).build_responder().unwrap()
}

pub fn snow_initiator(keypair: &Keypair) -> HandshakeState {
	snow_builder(keypair).build_initiator().unwrap()
}
