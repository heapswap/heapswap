use super::keys::Keypair;
use crate::arr;
use crate::*;
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

#[wasm_bindgen]
#[derive(Eq, PartialEq)]
pub enum NoiseRole {
	Initiator,
	Responder,
}

pub type NoiseKeypairString = String;

#[wasm_bindgen]
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
	fn new(
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

	pub fn initiator() -> Noise {
		let keypair = Keypair::random();
		Noise::initiator_from_keypair(keypair)
	}

	pub fn initiator_from_keypair(keypair: Keypair) -> Noise {
		let state = Builder::new(NOISE_PARAMS.clone())
			.local_private_key(keypair.private_key().v256().data())
			//.remote_public_key(keypair.public_key().data().data().data_u8())
			.build_initiator()
			.unwrap();

		Noise::new(NoiseRole::Initiator, keypair, state)
	}

	pub fn responder() -> Noise {
		let keypair = Keypair::random();
		Noise::responder_from_keypair(keypair)
	}

	pub fn responder_from_keypair(keypair: Keypair) -> Noise {
		let state = Builder::new(NOISE_PARAMS.clone())
			.local_private_key(keypair.private_key().v256().data())
			//.remote_public_key(keypair.public_key().data().data().data_u8())
			.build_responder()
			.unwrap();

		Noise::new(NoiseRole::Responder, keypair, state)
	}

	/**
	 * Getters
		*/
	pub fn keypair(&self) -> Keypair {
		self.keypair.clone()
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
			.local_private_key(self.keypair.private_key().v256().data())
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

#[wasm_bindgen]
impl Noise {
	/**
	 * Constructors
		*/
	#[wasm_bindgen(js_name = "initiator")]
	pub fn _js_initiator() -> Noise {
		Noise::initiator()
	}

	#[wasm_bindgen(js_name = "responder")]
	pub fn _js_responder() -> Noise {
		Noise::responder()
	}

	#[wasm_bindgen(js_name = "initiatorFromKeypair")]
	pub fn _js_initiator_from_keypair(keypair: Keypair) -> Noise {
		Noise::initiator_from_keypair(keypair)
	}

	#[wasm_bindgen(js_name = "responderFromKeypair")]
	pub fn _js_responder_from_keypair(keypair: Keypair) -> Noise {
		Noise::responder_from_keypair(keypair)
	}

	/**
	 * Getters
		*/
	#[wasm_bindgen(getter, js_name = "keypair")]
	pub fn _js_keypair(&self) -> Keypair {
		self.keypair()
	}

	/**
	 * Handshake
		*/
	#[wasm_bindgen(js_name = "handshakeStep1")]
	pub fn _js_handshake_step_1(&mut self) -> Uint8Array {
		self.handshake_step_1().unwrap().as_slice().into()
	}

	#[wasm_bindgen(js_name = "handshakeStep2")]
	pub fn _js_handshake_step_2(&mut self, message: Uint8Array) -> Uint8Array {
		self.handshake_step_2(&message.to_vec().as_slice())
			.unwrap()
			.as_slice()
			.into()
	}

	#[wasm_bindgen(js_name = "handshakeStep3")]
	pub fn _js_handshake_step_3(&mut self, message: Uint8Array) {
		self.handshake_step_3(&message.to_vec().as_slice()).unwrap();
	}

	/**
	 * Encrypt
		*/
	#[wasm_bindgen(js_name = "encrypt")]
	pub fn _js_encrypt(&mut self, data: Uint8Array) -> Uint8Array {
		self.encrypt(&data.to_vec().as_slice())
			.unwrap()
			.as_slice()
			.into()
	}

	/**
	 * Decrypt
		*/
	#[wasm_bindgen(js_name = "decrypt")]
	pub fn _js_decrypt(&mut self, data: Uint8Array) -> Uint8Array {
		self.decrypt(&data.to_vec().as_slice())
			.unwrap()
			.as_slice()
			.into()
	}
}
