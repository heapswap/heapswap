#![allow(unused)]
// extern crate serde;
// extern crate serde_json;
use poem_grpc::include_proto;
pub use prost::Message;
use bytes::{Bytes, BytesMut};
use poem_grpc::{Status, Code}	;

pub fn proto_serialize<T: Message>(message: T) -> Result<Bytes, Status> {
	let mut buf = BytesMut::with_capacity(message.encoded_len());
	message.encode(&mut buf).map_err(|_| Status::new(Code::Internal))?;
	Ok(buf.freeze())
}

pub fn proto_deserialize<T: Message + Default>(bytes: Bytes) -> Result<T, Status> {
	T::decode(&mut bytes.as_ref()).map_err(|_| Status::new(Code::Internal))
}


// include!(concat!(env!("OUT_DIR"), "/lib.rs"));
include_proto!("versioned_bytes");
