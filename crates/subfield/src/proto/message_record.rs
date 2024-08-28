use crate::*;

/**
 * GetRecord
 */
#[derive(Serialize, Deserialize, Clone)]
pub struct GetRecordRequest {
	pub routing_subkey: RoutingSubkey,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetRecordSuccess {
	pub routing_subkey: RoutingSubkey,
	pub record_bytes: Vec<u8>,
	pub signature: crypto::Signature,
}

#[derive(Debug)]
pub enum GetRecordFailure {
	Unknown = 0,
	Invalid = 1,
}

pub type GetRecordResponse = Result<GetRecordSuccess, GetRecordFailure>;

/**
 * PutRecord
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct PutRecordRequest {
	pub routing_subkey: RoutingSubkey,
	pub record_bytes: Vec<u8>,
	pub signature: crypto::Signature,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PutRecordSuccess {}

#[derive(Debug)]
pub enum PutRecordFailure {
	Unknown = 0,
	Invalid = 1,
}

pub type PutRecordResponse = Result<PutRecordSuccess, PutRecordFailure>;

/**
 * DeleteRecord
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct DeleteRecordRequest {
	pub routing_subkey: RoutingSubkey,
	pub signature: DeleteRecordSignature,
}

pub enum DeleteRecordSignature {
	Signer(Signature),
	Cosigner(Signature),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeleteRecordSuccess {}

#[derive(Debug)]
pub enum DeleteRecordFailure {
	Unknown = 0,
	Invalid = 1,
}

pub type DeleteRecordResponse = Result<DeleteRecordSuccess, DeleteRecordFailure>;