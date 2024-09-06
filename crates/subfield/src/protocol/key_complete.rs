use crate::*;
use std::hash::Hash;

pub type CompleteKeyField = V256;

// this is a special case of the base key type
// all fields are expected to be set
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, Deserialize)]
pub struct CompleteKey {
	pub signer: CompleteKeyField,
	pub cosigner: CompleteKeyField,
	pub tangent: CompleteKeyField,
}

impl SubfieldKey for CompleteKey {}

impl CompleteKey {
	/*
	 Conversions
	*/
	pub fn to_partial(&self) -> PartialKey {
		PartialKey {
			signer: Some(self.signer.clone()),
			cosigner: Some(self.cosigner.clone()),
			tangent: Some(self.tangent.clone()),
		}
	}

	pub fn from_partial(
		partial: PartialKey,
	) -> Result<CompleteKey, SubfieldError> {
		Ok(CompleteKey {
			signer: partial
				.signer
				.ok_or(SubfieldError::CompleteKeyMissingField)?,
			cosigner: partial
				.cosigner
				.ok_or(SubfieldError::CompleteKeyMissingField)?,
			tangent: partial
				.tangent
				.ok_or(SubfieldError::CompleteKeyMissingField)?,
		})
	}

	pub fn to_signer_routing_key(&self) -> RoutingKey {
		RoutingKey::Signer(self.to_partial())
	}

	pub fn to_cosigner_routing_key(&self) -> RoutingKey {
		RoutingKey::Cosigner(self.to_partial())
	}

	pub fn to_tangent_routing_key(&self) -> RoutingKey {
		RoutingKey::Tangent(self.to_partial())
	}

	/*
	 Hashing
	*/
	pub fn hash(&self) -> V256 {
		PartialKey::hash_concat(&[
			&Some(self.signer.clone()),
			&Some(self.cosigner.clone()),
			&Some(self.tangent.clone()),
		])
	}
}

/*
   Randomable
*/
impl Randomable for CompleteKey {
	fn random() -> Self {
		Self {
			signer: V256::random256(),
			cosigner: V256::random256(),
			tangent: V256::random256(),
		}
	}
}

/*
   Hash (for use in maps)
*/
impl Hash for CompleteKey {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.hash().hash(state)
	}
}
