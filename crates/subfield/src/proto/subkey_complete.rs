use crate::*;
use std::collections::HashSet;
use std::hash::Hash;

pub type CompleteSubkeyField = V256;

// this is a special case of the base subkey type
// all fields are expected to be set
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, Deserialize)]
pub struct CompleteSubkey {
	pub signer: CompleteSubkeyField,
	pub cosigner: CompleteSubkeyField,
	pub tangent: CompleteSubkeyField,
}

impl CompleteSubkey {
	
	/**
	 * Conversions
	*/
	pub fn to_partial(&self) -> PartialSubkey {
		PartialSubkey {
			signer: Some(self.signer.clone()),
			cosigner: Some(self.cosigner.clone()),
			tangent: Some(self.tangent.clone()),
		}
	}
	
	pub fn from_partial(partial: PartialSubkey) -> Result<CompleteSubkey, SubfieldError> {
		Ok(CompleteSubkey {
			signer: partial.signer.ok_or(SubfieldError::CompleteSubkeyMissingField)?,
			cosigner: partial.cosigner.ok_or(SubfieldError::CompleteSubkeyMissingField)?,
			tangent: partial.tangent.ok_or(SubfieldError::CompleteSubkeyMissingField)?,
		})
	}
	
	/**
	 * Hashing
	*/
	pub fn hash(&self) -> V256 {
		PartialSubkey::hash_concat(&[&Some(self.signer.clone()), &Some(self.cosigner.clone()), &Some(self.tangent.clone())])
	}		
}

/**
 * Randomable
*/
impl Randomable for CompleteSubkey {
	fn random() -> Self {
		Self {
			signer: V256::random256(),
			cosigner: V256::random256(),
			tangent: V256::random256(),
		}
	}
}

/**
 * Hash (for use in maps)
*/
impl Hash for CompleteSubkey {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.hash().hash(state)
	}
}
