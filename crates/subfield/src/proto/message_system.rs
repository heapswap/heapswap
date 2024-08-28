use crate::*;

/**
 * Ping
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct PingRequest {
	pub timestamp: DateTimeUtc,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PingSuccess {
	pub timestamp: DateTimeUtc,
}

pub enum PingFailure {
	Unknown = 0,
	Invalid = 1,
}

pub type PingResponse = Result<PingSuccess, PingFailure>;

/**
 * Echo
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct EchoRequest {
	pub message: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EchoSuccess {
	pub message: String,
}

pub enum EchoFailure {
	Unknown = 0,
	Invalid = 1,
}

pub type EchoResponse = Result<EchoSuccess, EchoFailure>;
