use crate::*;

/**
 * Subscribe
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscribeRequest {
	pub key: PartialKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscribeSuccess {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubscribeFailure {
	Unknown,
	Invalid,
	ServiceError(SubfieldError),
}

pub type SubscribeResponse = Result<SubscribeSuccess, SubscribeFailure>;

/**
 * Unsubscribe
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnsubscribeRequest {
	pub key: PartialKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnsubscribeSuccess {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UnsubscribeFailure {
	Unknown,
	Invalid,
	ServiceError(SubfieldError),
}

pub type UnsubscribeResponse = Result<UnsubscribeSuccess, UnsubscribeFailure>;
