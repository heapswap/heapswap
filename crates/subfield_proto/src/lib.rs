#![allow(unused)]
// extern crate serde;
// extern crate serde_json;
// use poem_grpc::include_proto;
pub use prost::Message;
use bytes::{Bytes, BytesMut};
// use poem_grpc::{Status, Code}	;
pub use prost::{EncodeError, DecodeError, };

pub use crate::message_pubsub::*;
pub use crate::message_record::*;
pub use crate::message_system::*;
pub use crate::versioned_bytes::*;
pub use crate::record::*;
// pub use crate::service::*;
pub use crate::subkey::*;
pub use crate::google::protobuf::*;


pub fn serialize<T: Message>(message: T) -> Result<Bytes, EncodeError> {
	let mut buf = BytesMut::with_capacity(message.encoded_len());
	message.encode(&mut buf);
	Ok(buf.freeze())
}

pub fn deserialize<T: Message + Default>(bytes: Bytes) -> Result<T, DecodeError> {
	T::decode(&mut bytes.as_ref())
}


include!(concat!(env!("OUT_DIR"), "/lib.rs"));
// include_proto!("versioned_bytes");


#[test]
fn test_proto_serialize() {
	let message = helloworld::HelloRequest {
		name: "Poem".into(),
	};
	let serialized = serialize(message.clone()).unwrap();
	let deserialized = deserialize::<helloworld::HelloRequest>(serialized.clone()).unwrap();
	assert_eq!(deserialized, message);
}