use crate::*;

#[async_trait]
pub trait SubfieldDispatchTrait{
	async fn dispatch_request<'a>(
		&self,
		request: SubfieldRequest,
		swarm: &mut MutexGuard<
			'a,
			libp2p::swarm::Swarm<swarm::SubfieldBehaviour>,
		>,
	) -> Result<(), SubfieldError> {
		Ok(())
	}
}


#[async_trait]
pub trait SubfieldHandlerTrait<MessageType: Clone + Send + 'static> {
	async fn handle_request<'a>(
		&self,
		routing_key: RoutingKey,
		request: MessageType,
		swarm: &mut MutexGuard<
			'a,
			libp2p::swarm::Swarm<swarm::SubfieldBehaviour>,
		>,
	) -> Result<(), SubfieldError> {
		Ok(())
	}
}
