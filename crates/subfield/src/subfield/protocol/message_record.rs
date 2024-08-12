use super::*;
use crate::*;

/**
 * Request
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubfieldRecordRequest {
	// CRUD
	Put(SignedPutRequest),
	Get(GetRequest),
	// Update(UpdateRequest),
	// Delete(DeleteRequest),
}

/**
 * Response
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubfieldRecordResponse {
	Put(PutResponse),
	Get(GetResponse),
	// Update(UpdateResponse),
	// Delete(DeleteResponse),
}

/**
 * Put
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
 * Get
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
