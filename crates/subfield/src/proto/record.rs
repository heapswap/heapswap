use crate::*;

#[derive(Serialize, Deserialize, Clone)]
pub enum RecordType {
	Simple = 0,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Record {
	pub record_type: RecordType,
	pub subkey: Subkey,

	pub is_encrypted: bool,
	hash_seed: VersionedBytes,
	data: VersionedBytes,

	pub created_at: DateTimeUtc,
	pub updated_at: DateTimeUtc,
}

pub enum RecordError {
	SubkeyIncomplete,
	KeypairNotSigner,
	InvalidSignature,
	GetRecordFailure(GetRecordFailure),
}

impl Record {
	pub fn to_put_record_requests(
		&self,
		subkey: Subkey,
		keypair: &Keypair,
	) -> Result<[PutRecordRequest; 3], RecordError> {
		// Subkey must be complete
		if !subkey.is_complete() {
			return Err(RecordError::SubkeyIncomplete);
		}

		// The keypair's public key must be equal to the subkey's signer
		if *keypair.public_key().v256() != subkey.clone().signer.unwrap() {
			return Err(RecordError::KeypairNotSigner);
		}

		let record_bytes = cbor_serialize(self).unwrap();

		let signature = keypair.sign(&record_bytes);

		let put_record_requests = [
			PutRecordRequest {
				routing_subkey: RoutingSubkey::Signer(subkey.clone()),
				record_bytes: record_bytes.clone(),
				signature: signature.clone(),
			},
			PutRecordRequest {
				routing_subkey: RoutingSubkey::Cosigner(subkey.clone()),
				record_bytes: record_bytes.clone(),
				signature: signature.clone(),
			},
			PutRecordRequest {
				routing_subkey: RoutingSubkey::Tangent(subkey.clone()),
				record_bytes: record_bytes.clone(),
				signature: signature.clone(),
			},
		];

		Ok(put_record_requests)
	}

	pub fn from_get_record_response(
		get_record_response: GetRecordResponse,
	) -> Result<Record, RecordError> {
		let success = get_record_response
			.map_err(|failure| RecordError::GetRecordFailure(failure))?;

		let subkey = flatten_routing_subkey(success.routing_subkey);

		// make sure the subkey is complete
		if !subkey.is_complete() {
			return Err(RecordError::SubkeyIncomplete);
		}

		// verify signature
		let public_key = PublicKey::new(subkey.signer.unwrap());
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
