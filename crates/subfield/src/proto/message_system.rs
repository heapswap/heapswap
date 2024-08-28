use crate::*;

/**
 * Ping
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct PingRequest {
	timestamp: DateTimeUtc,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PingSuccess {
	timestamp: DateTimeUtc,
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
	message: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EchoSuccess {
	message: String,
}

pub enum EchoFailure {
	Unknown = 0,
	Invalid = 1,
}

pub type EchoResponse = Result<EchoSuccess, EchoFailure>;
