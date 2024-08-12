use super::*;
use crate::*;

/**
 * Message
*/
pub trait _SubfieldMessageTrait {}
pub type SubfieldMessage = dyn _SubfieldMessageTrait;

/**
 * Request
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubfieldRequest {
	System(SubfieldSystemRequest),
	Record(SubfieldRecordRequest),
	Pubsub(SubfieldPubsubRequest),
}
impl _SubfieldMessageTrait for SubfieldRequest {}

/**
 * Response
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubfieldResponse {
	System(SubfieldSystemResponse),
	Record(SubfieldRecordResponse),
	Pubsub(SubfieldPubsubResponse),
}
impl _SubfieldMessageTrait for SubfieldResponse {}
