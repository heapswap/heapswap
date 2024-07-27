use super::*;
use crate::*;

/**
 * Subfield
*/
/*
pub type Topic = Option<u256::U256>;
pub type OptTopic = Option<Topic>;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subfield(
	pub OptTopic, // signer
	pub OptTopic, // cosigner
	pub OptTopic, // tangent
);
*/

/**
 * Request
*/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubfieldRequest {
	// CRUD
	Put(SignedPutRequest),
	Get(GetRequest),
	// Update(UpdateRequest),
	// Delete(DeleteRequest),
	// // Pubsub
	// Publish(PublishRequest),
	// Subscribe(SubscribeRequest),
	// Unsubscribe(UnsubscribeRequest),
}

/**
 * Response
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubfieldResponse {
	Put(PutResponse),
	Get(GetResponse),
}

/**
 * Entry
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubfieldEntry {
	pub data: Vec<u8>,
}

/**
 * Put Data
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PutRequest {
	// the destination to upload to
	pub topic_part: SubfieldTopicPart,
	// the full topic being uploaded
	pub topic_full: SubfieldTopic,

	// author - either signer or cosigner
	pub author: RemoteAuthor,

	// data
	pub entry: SubfieldEntry,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedPutRequest {
	// unsigned request
	pub put_request: PutRequest,

	// signature
	#[serde(with = "serde_bytes")]
	pub author_signature: crypto::Signature,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PutResponse {}

/**
 * Get Data
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRequest {
	// the destination being routed by
	pub topic_part: SubfieldTopicPart,
	// the full topic being searched for
	pub topic_full: SubfieldTopic,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse {
	pub entry: SubfieldEntry,
}
