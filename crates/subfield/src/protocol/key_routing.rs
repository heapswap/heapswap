use crate::*;

// a routing key is a wrapper around a partial key
// it is used to indicate which field of the key is being used for routing
// it must have at least the selected field set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingKey {
	Signer(PartialKey),
	Cosigner(PartialKey),
	Tangent(PartialKey),
}

impl SubfieldKey for RoutingKey {}

impl RoutingKey {
	pub fn random() -> Self {
		let key = PartialKey::random();
		let rand_int = rand::random::<u32>() % 3;
		match rand_int {
			0 => RoutingKey::Signer(key),
			1 => RoutingKey::Cosigner(key),
			2 => RoutingKey::Tangent(key),
			_ => unreachable!(),
		}
	}

	pub fn is_valid(&self) -> bool {
		match self {
			RoutingKey::Signer(key) => key.signer.is_some(),
			RoutingKey::Cosigner(key) => key.cosigner.is_some(),
			RoutingKey::Tangent(key) => key.tangent.is_some(),
		}
	}

	// get the internal key
	pub fn to_partial_key(&self) -> PartialKey {
		match self {
			RoutingKey::Signer(key) => key.clone(),
			RoutingKey::Cosigner(key) => key.clone(),
			RoutingKey::Tangent(key) => key.clone(),
		}
	}

	// get the internal key
	pub fn to_complete_key(&self) -> Result<CompleteKey, SubfieldError> {
		let partial_key = self.to_partial_key();
		CompleteKey::from_partial(partial_key)
	}

	// get the highlighted field of the key
	pub fn get_routing_field(&self) -> Result<CompleteKeyField, SubfieldError> {
		match self {
			RoutingKey::Signer(key) => key
				.signer
				.clone()
				.ok_or(SubfieldError::RoutingKeyMissingField),
			RoutingKey::Cosigner(key) => key
				.cosigner
				.clone()
				.ok_or(SubfieldError::RoutingKeyMissingField),
			RoutingKey::Tangent(key) => key
				.tangent
				.clone()
				.ok_or(SubfieldError::RoutingKeyMissingField),
		}
	}

	// pub fn to_key(&self) -> libp2p::kad::KBucketKey<Vec<u8>> {
	// 	self.get_routing_field().unwrap().to_key()
	// }
}
