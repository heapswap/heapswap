use super::*;
use crate::*;

/**
 * Request
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubfieldPubsubRequest {
	// Pubsub
	// Publish(PublishRequest),
	// Subscribe(SubscribeRequest),
	// Unsubscribe(UnsubscribeRequest),
}

/**
 * Response
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubfieldPubsubResponse {
	// Pubsub
	// Publish(PublishResponse),
	// Subscribe(SubscribeResponse),
	// Unsubscribe(UnsubscribeResponse),
}
