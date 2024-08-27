use crate::*;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubkeyError {
	InvalidProto,
	EncodeError,
	DecodeError,
	IncompleteSubkey,
}

pub type Keyfield = Option<V256>;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct Subkey {
	#[get = "pub"]
	signer: Keyfield,
	#[get = "pub"]
	cosigner: Keyfield,
	#[get = "pub"]
	tangent: Keyfield,
}

lazy_static! {
	static ref ZERO: V256 = V256::zero(0, 256);
}

impl Subkey {
	pub fn new(
		signer: Keyfield,
		cosigner: Keyfield,
		tangent: Keyfield,
	) -> Self {
		Self {
			signer,
			cosigner,
			tangent,
		}
	}

	/**
	 * Hash - for use in system
		*/

	fn hash_concat(hashes: &[&Option<V256>]) -> V256 {
		let concatenated: Vec<u8> = hashes
			.iter()
			.flat_map(|v| v.as_ref().unwrap_or(&ZERO).to_vec())
			.collect();
		crypto::hash(&concatenated)
	}

	pub fn hash(&self) -> V256 {
		Self::hash_concat(&[&self.signer, &self.cosigner, &self.tangent])
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

	pub fn is_complete(&self) -> bool {
		self.signer.is_some()
			&& self.cosigner.is_some()
			&& self.tangent.is_some()
	}
}

/**
 * Randomable
*/
impl Randomable for Subkey {
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
impl Hash for Subkey {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.hash().hash(state)
	}
}

/**
 * Protoable
*/

fn option_to_proto<T: Protoable<P, E>, P, E>(
	opt: Option<T>,
) -> Result<Option<P>, SubkeyError> {
	match opt {
		Some(t) => {
			Ok(Some(t.to_proto().map_err(|_| SubkeyError::InvalidProto)?))
		}
		None => Ok(None),
	}
}

fn key_to_proto(
	vb: &Option<V256>,
) -> Result<Option<proto::VersionedBytes>, SubkeyError> {
	option_to_proto::<V256, proto::VersionedBytes, VersionedBytesError>(
		vb.clone(),
	)
}

fn proto_to_option<T: Protoable<P, E>, P, E>(
	vb: Option<P>,
) -> Result<Option<T>, SubkeyError> {
	match vb {
		Some(p) => Ok(Some(
			T::from_proto(p).map_err(|_| SubkeyError::InvalidProto)?,
		)),
		None => Ok(None),
	}
}

fn proto_to_key(
	vb: &Option<proto::VersionedBytes>,
) -> Result<Option<V256>, SubkeyError> {
	proto_to_option::<V256, proto::VersionedBytes, VersionedBytesError>(
		vb.clone(),
	)
}

impl Protoable<proto::Subkey, SubkeyError> for Subkey {
	fn to_proto(&self) -> Result<proto::Subkey, SubkeyError> {
		Ok(proto::Subkey {
			signer: key_to_proto(&self.signer)?,
			cosigner: key_to_proto(&self.cosigner)?,
			tangent: key_to_proto(&self.tangent)?,
		})
	}

	fn from_proto(proto: proto::Subkey) -> Result<Self, SubkeyError> {
		Ok(Self {
			signer: proto_to_key(&proto.signer)?,
			cosigner: proto_to_key(&proto.cosigner)?,
			tangent: proto_to_key(&proto.tangent)?,
		})
	}

	fn to_proto_bytes(&self) -> Result<Bytes, SubkeyError> {
		proto::serialize(&self.to_proto()?)
			.map_err(|_| SubkeyError::EncodeError)
	}

	fn from_proto_bytes(bytes: Bytes) -> Result<Self, SubkeyError> {
		Self::from_proto(
			proto::deserialize(bytes).map_err(|_| SubkeyError::DecodeError)?,
		)
	}
}
