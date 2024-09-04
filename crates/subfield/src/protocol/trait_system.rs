use crate::*;

/*
	The system trait is used for internal swarm behaviour.
*/
#[async_trait]
pub trait SubfieldSystemTrait {
	
	async fn closest_global_peer(&self, key: &VersionedBytes) -> Result<PeerId, SubfieldError>;
	
	async fn event_loop(&self);
}