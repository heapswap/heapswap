use crate::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RecordType {
	Simple = 0,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record {
	pub record_type: RecordType,
	pub key: CompleteKey,

	pub is_encrypted: bool,
	hash_seed: VersionedBytes,
	data: VersionedBytes,

	pub created_at: DateTimeUtc,
	pub updated_at: DateTimeUtc,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RecordError {
	SerializationError,
	DeserializationError,
	KeyIncomplete,
	KeyMismatch,
	KeypairNotSigner,
	InvalidSignature,
	GetRecordFailure(GetRecordFailure),
	CryptoKeyError(CryptoKeyError),
	SubfieldError(SubfieldError),
}

impl Record {
	pub fn to_put_record_requests(
		&self,
		key: &CompleteKey,
		keypair: &Keypair,
	) -> Result<[SubfieldRequest; 3], RecordError> {
		// The keypair's public key must be equal to the key's signer
		if keypair.public_key().versioned_bytes() != &key.signer {
			return Err(RecordError::KeypairNotSigner);
		}

		let record_bytes = serialize(self).unwrap();

		let signature = keypair.sign(&record_bytes);

		let partial_key = PartialKey::from_complete(key.clone());

		let put_record_requests = [
			SubfieldRequest {
				routing_key: RoutingKey::Signer(partial_key.clone()),
				body: SubfieldRequestBody::PutRecord(PutRecordRequest {
					record_bytes: record_bytes.clone(),
					signature: signature.clone(),
				}),
			},
			SubfieldRequest {
				routing_key: RoutingKey::Cosigner(partial_key.clone()),
				body: SubfieldRequestBody::PutRecord(PutRecordRequest {
					record_bytes: record_bytes.clone(),
					signature: signature.clone(),
				}),
			},
			SubfieldRequest {
				routing_key: RoutingKey::Tangent(partial_key.clone()),
				body: SubfieldRequestBody::PutRecord(PutRecordRequest {
					record_bytes: record_bytes.clone(),
					signature: signature.clone(),
				}),
			},
		];

		Ok(put_record_requests)
	}

	pub fn from_get_record_response(
		get_record_response: GetRecordResponse,
	) -> Result<Record, RecordError> {
		let success = get_record_response
			.map_err(|failure| RecordError::GetRecordFailure(failure))?;

		let key = success
			.routing_key
			.to_complete_key()
			.map_err(|e| RecordError::SubfieldError(e))?;

		// verify signature
		let public_key = PublicKey::new(key.signer);
		match public_key.verify(&success.record_bytes, &success.signature) {
			Ok(is_valid) => {
				if !is_valid {
					return Err(RecordError::InvalidSignature);
				}
			}
			Err(e) => {
				return Err(RecordError::InvalidSignature);
			}
		}

		let record = deserialize::<Record>(&success.record_bytes).unwrap();
		Ok(record)
	}
}
