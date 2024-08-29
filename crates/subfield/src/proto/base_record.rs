use crate::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RecordType {
	Simple = 0,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record {
	pub record_type: RecordType,
	pub subkey: CompleteSubkey,

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
	SubkeyIncomplete,
	SubkeyMismatch,
	KeypairNotSigner,
	InvalidSignature,
	GetRecordFailure(GetRecordFailure),
	KeyError(KeyError),
	SubfieldError(SubfieldError),
}

impl Record {
	pub fn to_put_record_requests(
		&self,
		subkey: &CompleteSubkey,
		keypair: &Keypair,
	) -> Result<[SubfieldRequest; 3], RecordError> {
		
		// The keypair's public key must be equal to the subkey's signer
		if *keypair.public_key().v256() != subkey.clone().signer {
			return Err(RecordError::KeypairNotSigner);
		}

		let record_bytes = cbor_serialize(self).unwrap();

		let signature = keypair.sign(&record_bytes);
		
		let partial_subkey = PartialSubkey::from_complete(subkey.clone());

		let put_record_requests = [
			SubfieldRequest {
				subkey: RoutingSubkey::Signer(partial_subkey.clone()),
				body: SubfieldRequestBody::PutRecord(PutRecordRequest {
					record_bytes: record_bytes.clone(),
					signature: signature.clone(),
				}),
			},
			SubfieldRequest {
				subkey: RoutingSubkey::Cosigner(partial_subkey.clone()),
				body: SubfieldRequestBody::PutRecord(PutRecordRequest {
					record_bytes: record_bytes.clone(),
					signature: signature.clone(),
				}),
			},
			SubfieldRequest {
				subkey: RoutingSubkey::Tangent(partial_subkey.clone()),
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

		let subkey = success.routing_subkey.to_complete_subkey().map_err(|e| RecordError::SubfieldError(e))?;

		// verify signature
		let public_key = PublicKey::new(subkey.signer);
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

		let record = cbor_deserialize::<Record>(&success.record_bytes).unwrap();
		Ok(record)
	}
}
