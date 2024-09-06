pub use super::common::*;
use crate::*;
use std::convert::From;

use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use ed25519_dalek::SIGNATURE_LENGTH;
use ed25519_dalek::{
	Signature as DalekSignature, Verifier, VerifyingKey as DalekEdPublicKey,
};
use once_cell::sync::OnceCell;
use x25519_dalek::PublicKey as DalekXPublicKey;

#[derive(Clone, Getters, Serialize, Deserialize, Debug)]
#[wasm_bindgen]
pub struct PublicKey {
	versioned_bytes: V256, // edwards public key
	#[serde(skip)]
	ed: OnceCell<DalekEdPublicKey>,
	#[serde(skip)]
	x: OnceCell<DalekXPublicKey>,
}

/*
   PublicKey
*/
impl PublicKey {
	pub fn new(versioned_bytes: V256) -> PublicKey {
		PublicKey {
			versioned_bytes,
			ed: OnceCell::new(),
			x: OnceCell::new(),
		}
	}

	/*
	Getters
	*/
	pub fn ed(&self) -> &DalekEdPublicKey {
		self.ed.get_or_init(|| {
			DalekEdPublicKey::from_bytes(
				&self.versioned_bytes().data().as_slice().try_into().unwrap(),
			)
			.unwrap()
		})
	}

	pub fn x(&self) -> &DalekXPublicKey {
		self.x.get_or_init(|| {
			DalekXPublicKey::from(self.ed().to_montgomery().to_bytes())
		})
	}

	/*
	Operations
	*/

	pub fn verify(
		&self,
		message: &[u8],
		signature: &Signature,
	) -> Result<bool, CryptoKeyError> {
		match self.ed().verify(
			message,
			&DalekSignature::from(
				&<[_; SIGNATURE_LENGTH]>::try_from(signature.data().as_slice())
					.map_err(|_| CryptoKeyError::InvalidSignature)?,
			),
		) {
			Ok(_) => Ok(true),
			Err(_) => Ok(false),
		}
	}

	pub fn bigint(&self) -> &BigUint {
		self.versioned_bytes().bigint()
	}
}

impl HasV256 for PublicKey {
	fn versioned_bytes(&self) -> &V256 {
		&self.versioned_bytes
	}
}

/*
   Hash
*/
impl std::hash::Hash for PublicKey {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.versioned_bytes.hash(state);
	}
}

/*
   Stringable
*/
impl Stringable<CryptoKeyError> for PublicKey {
	fn to_string(&self) -> String {
		self.versioned_bytes.to_string()
	}

	fn from_string(string: &str) -> Result<Self, CryptoKeyError> {
		Ok(PublicKey::new(
			V256::from_string(string)
				.map_err(|_| CryptoKeyError::InvalidPublicKey)?,
		))
	}
}

/*
   Vecable
*/
impl Vecable<CryptoKeyError> for PublicKey {
	fn to_vec(&self) -> Vec<u8> {
		self.versioned_bytes().to_vec()
	}

	fn from_arr(arr: &[u8]) -> Result<Self, CryptoKeyError> {
		Ok(PublicKey::new(
			V256::from_arr(arr)
				.map_err(|_| CryptoKeyError::InvalidPublicKey)?,
		))
	}
}

/*
   Randomable (nonsense, only used for testing)
*/
impl Randomable for PublicKey {
	fn random() -> Self {
		PublicKey::new(V256::random256())
	}
}

/*
   Equality
*/
impl PartialEq for PublicKey {
	fn eq(&self, other: &Self) -> bool {
		self.versioned_bytes.version() == other.versioned_bytes.version()
			&& self.versioned_bytes.data() == other.versioned_bytes.data()
	}
}

impl Eq for PublicKey {}

impl PartialOrd for PublicKey {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.versioned_bytes().partial_cmp(&other.versioned_bytes())
	}
}

impl Ord for PublicKey {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.versioned_bytes().cmp(&other.versioned_bytes())
	}
}

/*
   Protoable
*/
/*
impl Protoable<subfield_proto::PublicKey, CryptoKeyError> for PublicKey {
	fn from_proto(proto: subfield_proto::PublicKey) -> Result<Self, CryptoKeyError> {
		Ok(PublicKey::new(
			V256::from_proto(subfield_proto::VersionedBytes {
				version: proto.version,
				data: proto.data.clone().into(),
			})
			.map_err(|_| CryptoKeyError::InvalidPublicKey)?,
		))
	}

	fn to_proto(&self) -> Result<subfield_proto::PublicKey, CryptoKeyError> {
		Ok(subfield_proto::PublicKey {
			version:  self.versioned_bytes.version(),
			data: self.versioned_bytes.data().clone().into(),
		})
	}

	fn from_proto_bytes(bytes: Bytes) -> Result<Self, CryptoKeyError> {
		Ok(Self::from_proto(
			proto::deserialize::<subfield_proto::PublicKey>(bytes).unwrap(),
		)
		.map_err(|_| CryptoKeyError::InvalidPublicKey)?)
	}

	fn to_proto_bytes(&self) -> Result<Bytes, CryptoKeyError> {
		Ok(proto::serialize::<subfield_proto::PublicKey>(
			&self.to_proto().map_err(|_| CryptoKeyError::InvalidPublicKey)?,
		)
		.map_err(|_| CryptoKeyError::InvalidPublicKey)?)
	}
}
*/


type Libp2pPublicKey = libp2p::identity::PublicKey;
type Libp2pEdPublicKey = libp2p::identity::ed25519::PublicKey;

impl Libp2pPublicKeyable<CryptoKeyError> for PublicKey {
	fn to_libp2p_public_key(
		&self,
	) -> Result<Libp2pEdPublicKey, CryptoKeyError> {
		let bytes: [u8; PUBLIC_KEY_LENGTH] =
			self.versioned_bytes().data().as_slice().try_into().unwrap();
		Ok(Libp2pEdPublicKey::try_from_bytes(&bytes).unwrap())
	}

	fn from_libp2p_public_key(
		public_key: Libp2pEdPublicKey,
	) -> Result<Self, CryptoKeyError> {
		let bytes: [u8; PUBLIC_KEY_LENGTH] = public_key.to_bytes();
		Ok(PublicKey::new(
			V256::from_arr(&bytes)
				.map_err(|_| CryptoKeyError::InvalidPublicKey)?,
		))
	}
}

impl Libp2pPeerIdable<CryptoKeyError> for PublicKey {
	fn to_libp2p_peer_id(&self) -> Result<libp2p::PeerId, CryptoKeyError> {
		let libp2p_key = self
			.to_libp2p_public_key()
			.map_err(|_| CryptoKeyError::InvalidPublicKey)?;
		let libp2p_public_key = Libp2pPublicKey::from(libp2p_key);
		Ok(libp2p::PeerId::from_public_key(&libp2p_public_key))
	}
}


#[wasm_bindgen]
impl PublicKey {
	/*
	Constructor
	*/
	#[wasm_bindgen(constructor)]
	pub fn _js_new(versioned_bytes: V256) -> Self {
		PublicKey::new(versioned_bytes)
	}

	/*
	Getters
	*/
	#[wasm_bindgen(getter, js_name = "ed")]
	pub fn _js_ed(&self) -> Uint8Array {
		self.ed().to_bytes().as_slice().into()
	}

	#[wasm_bindgen(getter, js_name = "x")]
	pub fn _js_x(&self) -> Uint8Array {
		self.x().to_bytes().as_slice().into()
	}

	/*
	Verify
	*/
	#[wasm_bindgen(js_name = "verify")]
	pub fn _js_verify(&self, message: &[u8], signature: &Signature) -> bool {
		self.verify(message, signature).unwrap()
	}

	/*
	Byteable
	*/
	#[wasm_bindgen(js_name = "toBytes")]
	pub fn _js_to_bytes(&self) -> Uint8Array {
		self.versioned_bytes()._js_to_bytes()
	}

	#[wasm_bindgen(js_name = "fromBytes")]
	pub fn _js_from_bytes(bytes: Uint8Array) -> Self {
		PublicKey::new(V256::_js_from_bytes(bytes))
	}

	/*
	Stringable
	*/
	#[wasm_bindgen(js_name = "toString")]
	pub fn _js_to_string(&self) -> String {
		self.versioned_bytes()._js_to_string()
	}

	#[wasm_bindgen(js_name = "fromString")]
	pub fn _js_from_string(string: &str) -> Self {
		PublicKey::new(V256::_js_from_string(string))
	}
}
