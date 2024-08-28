use crate::*;

/**
 * Subscribe
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct SubscribeRequest {
	pub subkey: Subkey,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubscribeSuccess {}

#[derive(Debug)]
pub enum SubscribeFailure {
	Unknown = 0,
	Invalid = 1,
}

pub type SubscribeResponse = Result<SubscribeSuccess, SubscribeFailure>;

/**
 * Unsubscribe
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct UnsubscribeRequest {
	pub subkey: Subkey,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UnsubscribeSuccess {}

#[derive(Debug)]
pub enum UnsubscribeFailure {
	Unknown = 0,
	Invalid = 1,
}

pub type UnsubscribeResponse = Result<UnsubscribeSuccess, UnsubscribeFailure>;
