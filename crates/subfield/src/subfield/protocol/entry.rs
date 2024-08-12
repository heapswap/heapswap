use super::*;
use crate::*;
/**
 * Entry
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubfieldEntry {
	pub data: Vec<u8>,
}
