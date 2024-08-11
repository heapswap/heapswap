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
	PutRecord(SignedPutRecordRequest),
	GetRecord(GetRecordRequest),
	// UpdateRecord(UpdateRecordRequest),
	// DeleteRecord(DeleteRecordRequest),
	
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
	PutRecord(PutRecordResponse),
	GetRecord(GetRecordResponse),
}

/**
 * Entry
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubfieldEntry {
	pub data: Vec<u8>,
}

/**
 * PutRecord
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PutRecordRequest {
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
pub struct SignedPutRecordRequest {
	// unsigned request
	pub put_request: PutRecordRequest,

	// signature
	#[serde(with = "serde_bytes")]
	pub author_signature: crypto::Signature,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PutRecordResponse {}

/**
 * GetRecord
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRecordRequest {
	// the destination being routed by
	pub topic_part: SubfieldTopicPart,
	// the full topic being searched for
	pub topic_full: SubfieldTopic,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRecordResponse {
	pub entry: SubfieldEntry,
}
