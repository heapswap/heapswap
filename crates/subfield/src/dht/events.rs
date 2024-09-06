use crate::*;

/*
	NetworkBehaviour <-> Swarm
*/
// The functions users of the swarm can call
#[async_trait]
pub trait FromSwarmToBehaviour {}

// The messages that the behaviour can send to the swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromBehaviourToSwarm {}


/*
	NetworkBehaviour <-> Connection Handler
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromBehaviourToHandler {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromHandlerToBehaviour {}


/*
	Client <-> Server
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromClientToServer {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromServerToClient {}

/*
	Server <-> Server
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromServerToServer {
	Echo
}