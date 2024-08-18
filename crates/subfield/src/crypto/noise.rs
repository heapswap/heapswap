use super::keys::Keypair;
use crate::arr;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
pub use snow::{
	params::NoiseParams, Builder, HandshakeState, Keypair as NoiseKeypair,
	TransportState,
};
use std::sync::Mutex;

pub type NoiseBuffer = Vec<u8>;

lazy_static! {
	pub static ref NOISE_PARAMS: NoiseParams =
		"Noise_NN_25519_ChaChaPoly_BLAKE2s".parse().unwrap();
}

// each chunk has 16 bytes of overhead
const CHUNK_SIZE: usize = 1024;
const OVERHEAD: usize = 16;
const CHUNK_SIZE_WITHOUT_OVERHEAD: usize = CHUNK_SIZE - OVERHEAD;
const CHUNK_SIZE_WITH_OVERHEAD: usize = CHUNK_SIZE + OVERHEAD;

#[derive(Debug, Clone, strum::Display)]
pub enum NoiseError {
	#[strum(serialize = "InvalidKey")]
	InvalidKey,
	#[strum(serialize = "InvalidMessage")]
	InvalidMessage,
	#[strum(serialize = "InvalidState")]
	InvalidState,
	#[strum(serialize = "InvalidBuffer")]
	InvalidBuffer,
	#[strum(serialize = "FailedToEncrypt")]
	FailedToEncrypt,
	#[strum(serialize = "FailedToDecrypt")]
	FailedToDecrypt,
}

#[derive(Eq, PartialEq)]
pub enum NoiseRole {
	Initiator,
	Responder,
}

pub type NoiseKeypairString = String;

pub struct Noise {
	role: NoiseRole,
	keypair: Keypair,
	//noise_keypair: NoiseKeypair,
	handshake: Mutex<HandshakeState>,
	transport: OnceCell<Mutex<TransportState>>,
	buffer: Mutex<NoiseBuffer>,
}

impl Noise {
	/**
	 * Constructors
		*/
	pub fn new(
		role: NoiseRole,
		keypair: Keypair,
		handshake_state: HandshakeState,
	) -> Self {
		Noise {
			role,
			keypair,
			handshake: Mutex::new(handshake_state),
			transport: OnceCell::new(),
			buffer: Mutex::new(vec![0u8; CHUNK_SIZE]),
		}
	}

	/**
	 * Getters
		*/
	pub fn keypair(&self) -> Keypair {
		self.keypair.clone()
	}

	/**
	 * Initiator
		*/
	pub fn initiator() -> Noise {
		let keypair = Keypair::random();
		Noise::initiator_from_keypair(keypair)
	}

	pub fn initiator_from_keypair(keypair: Keypair) -> Noise {
		let state = Builder::new(NOISE_PARAMS.clone())
			.local_private_key(keypair.private_key().data().data())
			//.remote_public_key(keypair.public_key().data().data().data_u8())
			.build_initiator()
			.unwrap();

		Noise::new(NoiseRole::Initiator, keypair, state)
	}

	/**
	 * Responder
		*/
	pub fn responder() -> Noise {
		let keypair = Keypair::random();
		Noise::responder_from_keypair(keypair)
	}

	pub fn responder_from_keypair(keypair: Keypair) -> Noise {
		let state = Builder::new(NOISE_PARAMS.clone())
			.local_private_key(keypair.private_key().data().data())
			//.remote_public_key(keypair.public_key().data().data().data_u8())
			.build_responder()
			.unwrap();

		Noise::new(NoiseRole::Responder, keypair, state)
	}

	/**
	 * Handshake
		*/

	// write a blank message to the buffer
	fn encrypt_handshake(&mut self) -> Result<NoiseBuffer, NoiseError> {
		let mut buffer = self.buffer.lock().unwrap();
		let handshake = self.handshake.get_mut().unwrap();
		let len = handshake
			.write_message(&[], &mut buffer)
			.map_err(|_| NoiseError::FailedToEncrypt)?;
		Ok(buffer[..len].to_vec())
	}

	// read a message from the buffer
	fn decrypt_handshake(
		&mut self,
		data: &[u8],
	) -> Result<NoiseBuffer, NoiseError> {
		let mut buffer = self.buffer.lock().unwrap();
		let handshake = self.handshake.get_mut().unwrap();
		let len = handshake
			.read_message(data, &mut buffer)
			.map_err(|_| NoiseError::FailedToDecrypt)?;
		Ok(buffer[..len].to_vec())
	}

	// Step 1
	// Initiator: Write the first handshake message
	pub fn handshake_step_1(&mut self) -> Result<NoiseBuffer, NoiseError> {
		self.encrypt_handshake()
	}

	// Step 2
	// Responder: Read the first handshake message
	// Responder: Write the second handshake message
	pub fn handshake_step_2(
		&mut self,
		message: &[u8],
	) -> Result<NoiseBuffer, NoiseError> {
		let _ = self.decrypt_handshake(message);
		let result = self.encrypt_handshake();
		self.into_transport_mode();
		result
	}

	// Step 3
	// Initiator: Read the second handshake message
	pub fn handshake_step_3(
		&mut self,
		message: &[u8],
	) -> Result<(), NoiseError> {
		let _ = self.decrypt_handshake(message)?;
		self.into_transport_mode();
		Ok(())
	}

	// into transport mode
	fn into_transport_mode(&mut self) {
		let handshake_option = self.handshake.get_mut().unwrap();
		// Replace the handshake with a dummy
		let dummy_handshake_state = Builder::new(NOISE_PARAMS.clone())
			.local_private_key(self.keypair.private_key().data().data())
			.build_initiator()
			.unwrap();
		let handshake =
			std::mem::replace(handshake_option, dummy_handshake_state);

		let transport = handshake.into_transport_mode().unwrap();

		let _ = self.transport.set(Mutex::new(transport));
	}

	/**
	 * Encrypt

	*/
	pub fn encrypt(&mut self, data: &[u8]) -> Result<NoiseBuffer, NoiseError> {
		let mut encrypted_data = Vec::new();

		let mut buffer = self.buffer.lock().unwrap();
		let mut transport = self.transport.get().unwrap().lock().unwrap();

		for chunk in data.chunks(CHUNK_SIZE_WITHOUT_OVERHEAD) {
			let len = transport
				.write_message(chunk, &mut buffer)
				.map_err(|_| NoiseError::FailedToEncrypt)?;
			encrypted_data.extend_from_slice(&buffer[..len]);
		}

		Ok(encrypted_data)
	}

	/**
	 * Decrypt
		*/
	pub fn decrypt(&mut self, data: &[u8]) -> Result<NoiseBuffer, NoiseError> {
		let mut decrypted_data = Vec::new();
		let mut buffer = self.buffer.lock().unwrap();
		let mut transport = self.transport.get().unwrap().lock().unwrap();

		for chunk in data.chunks(CHUNK_SIZE) {
			let len = transport
				.read_message(chunk, &mut buffer)
				.map_err(|_| NoiseError::FailedToDecrypt)?;
			decrypted_data.extend_from_slice(&buffer[..len]);
		}

		Ok(decrypted_data)
	}
}

#[test]
fn test_noise() {
	let mut initiator = Noise::initiator();
	let mut responder = Noise::responder();

	// handshake
	let initiator_message = initiator.handshake_step_1().unwrap();
	let responder_message =
		responder.handshake_step_2(&initiator_message).unwrap();
	let _ = initiator.handshake_step_3(&responder_message).unwrap();

	// encrypt from initiator to responder
	let data = b"hello world";
	let encrypted = initiator.encrypt(data).unwrap();
	let decrypted = responder.decrypt(&encrypted).unwrap();
	assert_eq!(data.to_vec(), decrypted);

	// encrypt from responder to initiator
	let data = b"hello world";
	let encrypted = responder.encrypt(data).unwrap();
	let decrypted = initiator.decrypt(&encrypted).unwrap();
	assert_eq!(data.to_vec(), decrypted);
}
