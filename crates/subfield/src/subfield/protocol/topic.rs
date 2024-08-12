use super::*;
use crate::*;
use getset::{Getters, Setters};

/**
 * A Subtopic represents one of the three fields that make up a SubfieldTopic
*/
pub type Subtopic = Option<Versioned>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubfieldTopicError {
	NoSigner,
	NoCosigner,
	NoTangent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubfieldTopicPart {
	Signer,
	Cosigner,
	Tangent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters)]
pub struct SubfieldTopic {
	#[getset(get = "pub")]
	signer: Subtopic,
	#[getset(get = "pub")]
	cosigner: Subtopic,
	#[getset(get = "pub")]
	tangent: Subtopic,
}

impl SubfieldTopic {
	/*
	 * Constructors
		*/
	pub fn new() -> Self {
		Self {
			signer: None,
			cosigner: None,
			tangent: None,
		}
	}

	pub fn with_signer(&mut self, signer: U256) -> &mut Self {
		self.signer = Some(signer);
		self
	}

	pub fn with_cosigner(&mut self, cosigner: U256) -> &mut Self {
		self.cosigner = Some(cosigner);
		self
	}

	pub fn with_tangent(&mut self, tangent: U256) -> &mut Self {
		self.tangent = Some(tangent);
		self
	}

	/*
	 * Operations
		*/

	// Check if the entry has no fields
	pub fn is_empty(&self) -> bool {
		self.signer().is_none()
			&& self.cosigner().is_none()
			&& self.tangent().is_none()
	}

	// Check if the entry has at least one of the three fields
	pub fn is_some(&self) -> bool {
		!self.is_empty()
	}

	// Check if the entry has a signer, cosigner, and tangent
	pub fn is_complete(&self) -> Result<(), SubfieldTopicError> {
		if self.signer().is_none() {
			return Err(SubfieldTopicError::NoSigner);
		}
		if self.cosigner().is_none() {
			return Err(SubfieldTopicError::NoCosigner);
		}
		if self.tangent().is_none() {
			return Err(SubfieldTopicError::NoTangent);
		}
		Ok(())
	}

	// hash the whole subfield
	pub fn hash(&self) -> U256 {
		let mut fields: Vec<&[u8]> = Vec::new();

		// signer, cosigner, and tangent are either unpacked U256 or [0;32]
		for method in [Self::signer, Self::cosigner, Self::tangent].iter() {
			if let Some(value) = method(self) {
				fields.push(value.data_u8().as_ref());
			} else {
				fields.push(&[0; 32]);
			}
		}

		crypto::hash(arr::concat(&fields).as_ref())
	}

	// return a list of the hashes of all possible combinations of the subfield
	pub fn hashes(&self) -> Vec<U256> {
		let mut result = Vec::new();

		let zero = &Some(U256::zero());
		let signer = self.signer();
		let cosigner = self.cosigner();
		let tangent = self.tangent();

		for (a, b, c) in &[
			// singles - hash(signer), hash(cosigner), hash(tangent)
			(signer, zero, zero),
			(zero, cosigner, zero),
			(zero, zero, tangent),
			// doubles - hash(all possible combinations of two fields)
			(signer, cosigner, zero),
			(signer, zero, tangent),
			(zero, cosigner, tangent),
			// triple - hash(signer, tangent, cosigner)
			(signer, cosigner, tangent),
		] {
			if let (Some(a), Some(b), Some(c)) =
				(a.as_ref(), b.as_ref(), c.as_ref())
			{
				result.push(crypto::hash(
					arr::concat(&[a.data_u8(), b.data_u8(), c.data_u8()])
						.as_ref(),
				));
			}
		}

		result
	}

	pub fn parts(&self) -> Vec<(SubfieldTopicPart, Subtopic)> {
		vec![
			(SubfieldTopicPart::Signer, self.signer().clone()),
			(SubfieldTopicPart::Cosigner, self.cosigner().clone()),
			(SubfieldTopicPart::Tangent, self.tangent().clone()),
		]
	}
}
