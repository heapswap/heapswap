use crate::*;
use super::super::*;

#[derive(Debug)]
pub enum NodeError {
	InvalidSwarmConfig,
}




pub struct Node {
	swarm: Arc<Mutex<swarm::SubfieldSwarm>>,
	swarm_config: swarm::SubfieldSwarmConfig,
}

impl Node {
	pub async fn new(swarm_config: swarm::SubfieldSwarmConfig) -> Result<Self, NodeError> {
		let swarm: Arc<Mutex<libp2p::Swarm<SubfieldBehaviour>>> = Arc::new(Mutex::new(swarm::create_swarm(swarm_config.clone()).await.map_err(
			|e| NodeError::InvalidSwarmConfig
		)?));
		Ok(Self { swarm, swarm_config })
	}
	
	pub fn listen(&self) {}
}