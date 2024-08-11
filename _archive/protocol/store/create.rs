use super::*;
use crate::*;

pub async fn create(config: SubfieldStoreConfig) -> EResult<SubfieldStore> {
	return EOk(SubfieldStore::new(config).await?);
}
