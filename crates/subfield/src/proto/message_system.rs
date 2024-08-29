use crate::*;

/**
 * Ping
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PingRequest {
	pub subkey: RoutingSubkey,
	pub timestamp: DateTimeUtc,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PingSuccess {
	pub timestamp: DateTimeUtc,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PingFailure {
	Unknown,
	Invalid,
	ServiceError(SubfieldError)
}

pub type PingResponse = Result<PingSuccess, PingFailure>;

/**
 * Echo
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EchoRequest {
	pub subkey: RoutingSubkey,
	pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EchoSuccess {
	pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EchoFailure {
	Unknown,
	Invalid,
	ServiceError(SubfieldError)
}

pub type EchoResponse = Result<EchoSuccess, EchoFailure>;
