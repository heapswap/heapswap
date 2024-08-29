use crate::*;
use std::collections::HashSet;
use std::hash::Hash;

// a routing subkey is a wrapper around a partial subkey
// it is used to indicate which field of the subkey is being used for routing
// it must have at least the selected field set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingSubkey {
	Signer(PartialSubkey),
	Cosigner(PartialSubkey),
	Tangent(PartialSubkey),
}


impl RoutingSubkey {
	pub fn random() -> Self {
		let subkey = PartialSubkey::random();
		let rand_int = rand::random::<u32>() % 3;
		match rand_int {
			0 => RoutingSubkey::Signer(subkey),
			1 => RoutingSubkey::Cosigner(subkey),
			2 => RoutingSubkey::Tangent(subkey),
			_ => unreachable!(),
		}
	}
	
	pub fn is_valid(&self) -> bool {
		match self {
			RoutingSubkey::Signer(subkey) => subkey.signer.is_some(),
			RoutingSubkey::Cosigner(subkey) => subkey.cosigner.is_some(),
			RoutingSubkey::Tangent(subkey) => subkey.tangent.is_some(),
		}
	}
	
	
	// get the internal subkey
	pub fn to_partial_subkey(&self) -> PartialSubkey {
		match self {
			RoutingSubkey::Signer(subkey) => subkey.clone(),
			RoutingSubkey::Cosigner(subkey) => subkey.clone(),
			RoutingSubkey::Tangent(subkey) => subkey.clone(),
		}
	}
	
	// get the internal subkey
	pub fn to_complete_subkey(&self) -> Result<CompleteSubkey, SubfieldError> {
		let partial_subkey = self.to_partial_subkey();
		CompleteSubkey::from_partial(partial_subkey)
	}
	
	// get the highlighted field of the subkey
	pub fn get_routing_field(&self) -> Result<CompleteSubkeyField, SubfieldError> {
		match self {
			RoutingSubkey::Signer(subkey) => subkey.signer.clone().ok_or(SubfieldError::RoutingSubkeyMissingField),
			RoutingSubkey::Cosigner(subkey) => subkey.cosigner.clone().ok_or(SubfieldError::RoutingSubkeyMissingField),
			RoutingSubkey::Tangent(subkey) => subkey.tangent.clone().ok_or(SubfieldError::RoutingSubkeyMissingField),
		}
	}
}