// use subfield_proto::signed_record;

use crate::*;
use std::collections::HashSet;
use std::hash::Hash;

pub type PartialKeyField = Option<CompleteKeyField>;

// this is the base key type
// all fields are optional, but at least one is expected to be set
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, Deserialize)]
pub struct PartialKey {
	pub signer: PartialKeyField,
	pub cosigner: PartialKeyField,
	pub tangent: PartialKeyField,
}

impl SubfieldKey for PartialKey {}

impl PartialKey {
	/*
	 Conversions
	*/

	pub fn is_complete(&self) -> bool {
		self.signer.is_some()
			&& self.cosigner.is_some()
			&& self.tangent.is_some()
	}

	pub fn to_complete(&self) -> Result<CompleteKey, KeyError> {
		if !self.is_complete() {
			return Err(KeyError::IncompleteKey);
		}
		Ok(CompleteKey {
			signer: self.signer.clone().unwrap(),
			cosigner: self.cosigner.clone().unwrap(),
			tangent: self.tangent.clone().unwrap(),
		})
	}

	pub fn from_complete(complete: CompleteKey) -> PartialKey {
		PartialKey {
			signer: Some(complete.signer),
			cosigner: Some(complete.cosigner),
			tangent: Some(complete.tangent),
		}
	}

	/*
	 Hashing
	*/

	pub fn hash(&self) -> V256 {
		Self::hash_concat(&[&self.signer, &self.cosigner, &self.tangent])
	}

	// hash multiple keys put together
	pub fn hash_concat(hashes: &[&Option<V256>]) -> V256 {
		let concatenated: Vec<u8> = hashes
			.iter()
			.flat_map(|v| v.as_ref().unwrap_or(&ZERO_V256).to_vec())
			.collect();
		crypto::hash(&concatenated)
	}

	// get all the combinations of a key, for use when publishing to pubsub
	pub fn hash_combinations(&self) -> Result<Vec<V256>, KeyError> {
		// // pubsub publishing requires a complete key
		// if !self.is_complete() {
		// 	return Err(KeyError::IncompleteKey);
		// }

		let mut res = HashSet::new();

		for i in [&self.signer, &None] {
			for j in [&self.cosigner, &None] {
				for k in [&self.tangent, &None] {
					if i.is_some() || j.is_some() || k.is_some() {
						res.insert(Self::hash_concat(&[i, j, k]));
					}
				}
			}
		}

		Ok(res.into_iter().collect())
	}

	/*
	// build the 3 get record requests to get a key
	pub fn to_get_record_requests(
		&self,
	) -> Result<[proto::GetRecordRequest; 3], KeyError> {
		// must be complete
		if !self.is_complete() {
			return Err(KeyError::IncompleteKey);
		}

		let requests = SUBKEY_FIELDS.map(|field| {
			proto::GetRecordRequest {
				key: Some(self.to_proto().unwrap()),
				field: field.into(),
			}
		});

		Ok(requests)
	}

	pub fn signer_is_signer(&self, signer: crypto::Keypair) -> Result<bool, KeyError> {
		if signer.public_key().v256() == self.signer.clone().unwrap().v256() {
			Ok(true)
		} else if signer.public_key().v256() == self.cosigner.clone().unwrap().v256() {
			Ok(false)
		} else {
			Err(KeyError::RequiresEitherSignerOrCosigner)
		}
	}

	pub fn to_put_record_requests(
		&self,
		signer: &crypto::Keypair,
		record: proto::Record,
	) -> Result<[proto::PutRecordRequest; 3], KeyError> {
		// must be complete
		if !self.is_complete() {
			return Err(KeyError::IncompleteKey);
		}

		let record_bytes = proto::serialize(&record).map_err(|_| KeyError::EncodeError)?.to_vec();

		let mut signature: Option<signed_record::Signature>;
		if self.signer_is_signer(signer)? {
			let sig = signer.sign(&record_bytes).to_proto().map_err(|_| KeyError::SignatureError)?;
			signature = Some(signed_record::Signature::SignerSignature(sig));
		} else {
			let sig = signer.sign(&record_bytes).to_proto().map_err(|_| KeyError::SignatureError)?;
			signature = Some(signed_record::Signature::CosignerSignature(sig));
		}

		let signed_record = Some(proto::SignedRecord {
			signature,
			record_bytes,
		});

		let key = Some(self.to_proto().map_err(|e| KeyError::InvalidProto)?);

		let requests = SUBKEY_FIELDS.map(|field| {
			proto::PutRecordRequest {
				key,
				signed_record,
				field: field.into(),
			}
		});

		Ok(requests)
	}
	*/
}

/**
 * Randomable
*/
impl Randomable for PartialKey {
	fn random() -> Self {
		Self {
			signer: Some(V256::random256()),
			cosigner: Some(V256::random256()),
			tangent: Some(V256::random256()),
		}
	}
}

/**
 * Hash (for use in maps)
*/
impl Hash for PartialKey {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.hash().hash(state)
	}
}
