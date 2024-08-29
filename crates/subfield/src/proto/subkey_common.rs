use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubkeyError {
	InvalidProto,
	EncodeError,
	DecodeError,
	IncompleteSubkey,
	SignatureError,
	RequiresEitherSignerOrCosigner,
}

lazy_static! {
	pub static ref ZERO_V256: V256 = V256::zero(0, 256);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
pub enum SubkeyFieldEnum {
	Signer = 0,
	Cosigner = 1,
	Tangent = 2,
}

pub const SUBKEY_FIELDS: [SubkeyFieldEnum; 3] = [
	SubkeyFieldEnum::Signer,
	SubkeyFieldEnum::Cosigner,
	SubkeyFieldEnum::Tangent,
];