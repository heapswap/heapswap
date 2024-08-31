use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyError {
	InvalidProto,
	EncodeError,
	DecodeError,
	IncompleteKey,
	SignatureError,
	RequiresEitherSignerOrCosigner,
}

pub trait SubfieldKey {}

lazy_static! {
	pub static ref ZERO_V256: V256 = V256::zero(0, 256);
}

#[derive(
	Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display,
)]
pub enum KeyFieldEnum {
	Signer = 0,
	Cosigner = 1,
	Tangent = 2,
}

pub const SUBKEY_FIELDS: [KeyFieldEnum; 3] = [
	KeyFieldEnum::Signer,
	KeyFieldEnum::Cosigner,
	KeyFieldEnum::Tangent,
];
