use crate::*;
pub use proto::subfield_request::RequestType as SubfieldRequestType;
pub use proto::subfield_response::ResponseType as SubfieldResponseType;
use serde::{
	de::{self, Deserialize, Deserializer, Visitor},
	ser::Error,
};

/**
 * SubfieldRequest
*/

#[derive(Debug, Clone)]
pub struct SubfieldRequest {
	pub proto: proto::SubfieldRequest,
}

impl SubfieldRequest {
	pub fn new(proto: proto::subfield_request::RequestType) -> Self {
		Self {
			proto: proto::SubfieldRequest {
				request_type: Some(proto),
			},
		}
	}

	pub fn proto(&self) -> &proto::SubfieldRequest {
		&self.proto
	}

	pub fn proto_type(&self) -> &proto::subfield_request::RequestType {
		&self.proto.request_type.as_ref().unwrap()
	}
}

impl Serialize for SubfieldRequest {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_bytes(
			proto::serialize(self.proto())
				.map_err(|e| S::Error::custom(e))?
				.as_ref(),
		)
	}
}

pub struct SubfieldRequestVisitor;

impl<'de> Visitor<'de> for SubfieldRequestVisitor {
	type Value = SubfieldRequest;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a byte array representing a SubfieldRequest")
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		let proto = proto::deserialize(Bytes::from(v.to_vec()))
			.map_err(|e| E::custom(e))?;
		Ok(SubfieldRequest { proto })
	}
}

impl<'de> Deserialize<'de> for SubfieldRequest {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_bytes(SubfieldRequestVisitor)
	}
}

/**
 * SubfieldResponse
*/

#[derive(Debug)]
pub struct SubfieldResponse {
	proto: proto::SubfieldResponse,
}

impl SubfieldResponse {
	pub fn new(proto: proto::subfield_response::ResponseType) -> Self {
		Self {
			proto: proto::SubfieldResponse {
				response_type: Some(proto),
			},
		}
	}

	pub fn proto(&self) -> &proto::SubfieldResponse {
		&self.proto
	}

	pub fn proto_type(&self) -> &proto::subfield_response::ResponseType {
		&self.proto.response_type.as_ref().unwrap()
	}
}

impl Serialize for SubfieldResponse {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_bytes(
			proto::serialize(self.proto())
				.map_err(|e| S::Error::custom(e))?
				.as_ref(),
		)
	}
}

pub struct SubfieldResponseVisitor;

impl<'de> Visitor<'de> for SubfieldResponseVisitor {
	type Value = SubfieldResponse;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a byte array representing a SubfieldResponse")
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		let proto = proto::deserialize::<proto::SubfieldResponse>(Bytes::from(
			v.to_vec(),
		))
		.map_err(|e| E::custom(e))?;
		Ok(SubfieldResponse { proto })
	}
}

impl<'de> Deserialize<'de> for SubfieldResponse {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_bytes(SubfieldResponseVisitor)
	}
}
