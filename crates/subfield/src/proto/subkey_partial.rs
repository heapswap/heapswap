// use subfield_proto::signed_record;

use crate::*;
use std::collections::HashSet;
use std::hash::Hash;



pub type PartialSubkeyField = Option<CompleteSubkeyField>;

// this is the base subkey type
// all fields are optional, but at least one is expected to be set
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, Deserialize)]
pub struct PartialSubkey {
	pub signer: PartialSubkeyField,
	pub cosigner: PartialSubkeyField,
	pub tangent: PartialSubkeyField,
}

impl PartialSubkey {
	
	/**
	 * Conversions
	*/
	
	pub fn is_complete(&self) -> bool {
		self.signer.is_some()
			&& self.cosigner.is_some()
			&& self.tangent.is_some()
	}
	
	pub fn to_complete(&self) -> Result<CompleteSubkey, SubkeyError> {
		if !self.is_complete() {
			return Err(SubkeyError::IncompleteSubkey);
		}
		Ok(CompleteSubkey {
			signer: self.signer.clone().unwrap(),
			cosigner: self.cosigner.clone().unwrap(),
			tangent: self.tangent.clone().unwrap(),
		})
	}

	/**
	 * Hashing
	*/

	pub fn hash(&self) -> V256 {
		Self::hash_concat(&[&self.signer, &self.cosigner, &self.tangent])
	}

	// hash multiple subkeys put together
	pub fn hash_concat(hashes: &[&Option<V256>]) -> V256 {
		let concatenated: Vec<u8> = hashes
			.iter()
			.flat_map(|v| v.as_ref().unwrap_or(&ZERO_V256).to_vec())
			.collect();
		crypto::hash(&concatenated)
	}

	// get all the combinations of a subkey, for use when publishing to pubsub
	pub fn hash_combinations(&self) -> Result<Vec<V256>, SubkeyError> {
		// // pubsub publishing requires a complete subkey
		// if !self.is_complete() {
		// 	return Err(SubkeyError::IncompleteSubkey);
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
	// build the 3 get record requests to get a subkey
	pub fn to_get_record_requests(
		&self,
	) -> Result<[proto::GetRecordRequest; 3], SubkeyError> {
		// must be complete
		if !self.is_complete() {
			return Err(SubkeyError::IncompleteSubkey);
		}

		let requests = SUBKEY_FIELDS.map(|field| {
			proto::GetRecordRequest {
				subkey: Some(self.to_proto().unwrap()),
				field: field.into(),
			}
		});

		Ok(requests)
	}

	pub fn signer_is_signer(&self, signer: crypto::Keypair) -> Result<bool, SubkeyError> {
		if signer.public_key().v256() == self.signer.clone().unwrap().v256() {
			Ok(true)
		} else if signer.public_key().v256() == self.cosigner.clone().unwrap().v256() {
			Ok(false)
		} else {
			Err(SubkeyError::RequiresEitherSignerOrCosigner)
		}
	}

	pub fn to_put_record_requests(
		&self,
		signer: &crypto::Keypair,
		record: proto::Record,
	) -> Result<[proto::PutRecordRequest; 3], SubkeyError> {
		// must be complete
		if !self.is_complete() {
			return Err(SubkeyError::IncompleteSubkey);
		}

		let record_bytes = proto::serialize(&record).map_err(|_| SubkeyError::EncodeError)?.to_vec();

		let mut signature: Option<signed_record::Signature>;
		if self.signer_is_signer(signer)? {
			let sig = signer.sign(&record_bytes).to_proto().map_err(|_| SubkeyError::SignatureError)?;
			signature = Some(signed_record::Signature::SignerSignature(sig));
		} else {
			let sig = signer.sign(&record_bytes).to_proto().map_err(|_| SubkeyError::SignatureError)?;
			signature = Some(signed_record::Signature::CosignerSignature(sig));
		}

		let signed_record = Some(proto::SignedRecord {
			signature,
			record_bytes,
		});

		let subkey = Some(self.to_proto().map_err(|e| SubkeyError::InvalidProto)?);

		let requests = SUBKEY_FIELDS.map(|field| {
			proto::PutRecordRequest {
				subkey,
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
impl Randomable for PartialSubkey {
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
impl Hash for PartialSubkey {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.hash().hash(state)
	}
}
