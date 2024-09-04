use crate::*;
use bytes::Bytes;

// Able to be randomly generated
pub trait Randomable: Sized {
	fn random() -> Self;
}

// Able to be randomly generated
pub trait RandomLengthable: Sized {
	fn random_length(length: usize) -> Self;
}

/*
// Able to be converted to and from bytes
pub trait Byteable<E>: Sized {
	fn to_bytes(&self) -> Bytes;
	fn from_bytes(bytes: Bytes) -> Result<Self, E>;
}
*/

// Able to be converted to and from a vec
pub trait Vecable<E>: Sized {
	fn to_vec(&self) -> Vec<u8>;
	fn from_arr(arr: &[u8]) -> Result<Self, E>;
}

// Able to be converted to and from a string
pub trait Stringable<E>: Sized {
	fn to_string(&self) -> String;
	fn from_string(string: &str) -> Result<Self, E>;
}

/*
// Able to be converted to and from a proto
pub trait Protoable<T, E>: Sized {
	fn to_proto(&self) -> Result<T, E>;
	fn from_proto(proto: T) -> Result<Self, E>;
	fn to_proto_bytes(&self) -> Result<Bytes, E>;
	fn from_proto_bytes(bytes: Bytes) -> Result<Self, E>;
}
*/

/*
   Has data of 32 bytes
*/
pub trait HasV256 {
	fn v256(&self) -> &V256;
}



pub trait Libp2pKeypairable<E>: Sized {
	fn to_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair, E>;
	fn from_libp2p_keypair(
		keypair: libp2p::identity::Keypair,
	) -> Result<Self, E>;
}

type Libp2pPublicKey = libp2p::identity::ed25519::PublicKey;
pub trait Libp2pPublicKeyable<E>: Sized {
	fn to_libp2p_public_key(&self) -> Result<Libp2pPublicKey, E>;
	fn from_libp2p_public_key(public_key: Libp2pPublicKey) -> Result<Self, E>;
}

pub trait Libp2pPeerIdable<E>: Sized {
	fn to_libp2p_peer_id(&self) -> Result<libp2p::PeerId, E>;
	// fn from_libp2p_peer_id(peer_id: libp2p::PeerId) -> Result<Self, E>;
}
