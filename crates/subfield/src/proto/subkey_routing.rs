use crate::*;
use std::collections::HashSet;
use std::hash::Hash;

// a routing subkey is a wrapper around a complete subkey
// it is used to indicate which field of the subkey is being used for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingSubkey {
	Signer(CompleteSubkey),
	Cosigner(CompleteSubkey),
	Tangent(CompleteSubkey),
}


impl RoutingSubkey {
	pub fn random() -> Self {
		let subkey = CompleteSubkey::random();
		let rand_int = rand::random::<u32>() % 3;
		match rand_int {
			0 => RoutingSubkey::Signer(subkey),
			1 => RoutingSubkey::Cosigner(subkey),
			2 => RoutingSubkey::Tangent(subkey),
			_ => unreachable!(),
		}
	}
	
	// get the internal subkey
	pub fn to_complete_subkey(&self) -> CompleteSubkey {
		match self {
			RoutingSubkey::Signer(subkey) => subkey.clone(),
			RoutingSubkey::Cosigner(subkey) => subkey.clone(),
			RoutingSubkey::Tangent(subkey) => subkey.clone(),
		}
	}
	
	// get the highlighted field of the subkey
	pub fn get_only_routing_field(&self) -> VersionedBytes {
		match self {
			RoutingSubkey::Signer(subkey) => subkey.signer.clone(),
			RoutingSubkey::Cosigner(subkey) => subkey.cosigner.clone(),
			RoutingSubkey::Tangent(subkey) => subkey.tangent.clone(),
		}
	}
}