use crate::*;

/*
   GetRecord
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetRecordRequest {
	pub routing_key: RoutingKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetRecordSuccess {
	pub routing_key: RoutingKey,
	pub record_bytes: Vec<u8>,
	pub signature: crypto::Signature,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GetRecordFailure {
	Unknown,
	Invalid,
	ServiceError(SubfieldError),
}

pub type GetRecordResponse = Result<GetRecordSuccess, GetRecordFailure>;

/*
   PutRecord
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PutRecordRequest {
	pub record_bytes: Vec<u8>,
	pub signature: crypto::Signature,
}

impl PutRecordRequest {
	pub fn verify(
		&self,
		routing_key: RoutingKey,
	) -> Result<(CompleteKey, Record), RecordError> {
		let key = routing_key
			.to_complete_key()
			.map_err(|e| RecordError::SubfieldError(e))?;
		let record: Record = cbor_deserialize(&self.record_bytes)
			.map_err(|e| RecordError::DeserializationError)?;

		// routing key must be the same as the internal key
		if key != record.key {
			return Err(RecordError::KeyMismatch);
		}

		// verify the signature
		let public_key = crypto::PublicKey::new(key.signer.clone());
		match public_key.verify(&self.record_bytes, &self.signature) {
			Ok(true) => Ok((key, record)),
			Ok(false) => Err(RecordError::InvalidSignature),
			Err(e) => Err(RecordError::CryptoKeyError(e)),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PutRecordSuccess {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PutRecordFailure {
	Unknown,
	Invalid,
	NoPeersConnected,
	ServiceError(SubfieldError),
	RecordError(RecordError),
}

pub type PutRecordResponse = Result<PutRecordSuccess, PutRecordFailure>;

/*
   DeleteRecord
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteRecordRequest {
	pub signature: DeleteRecordSignature,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeleteRecordSignature {
	Signer(Signature),
	Cosigner(Signature),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteRecordSuccess {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeleteRecordFailure {
	Unknown,
	Invalid,
	ServiceError(SubfieldError),
}

pub type DeleteRecordResponse =
	Result<DeleteRecordSuccess, DeleteRecordFailure>;
