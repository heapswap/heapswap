use crate::*;

#[async_trait]
pub trait SubfieldHandlerTrait<MessageType: Clone + Send + 'static> {
	async fn handle_request<'a>(
		&self,
		request_id: InboundRequestId,
		request: MessageType,
		response_channel: ResponseChannel<SubfieldResponse>,
		swarm: &mut MutexGuard<
			'a,
			libp2p::swarm::Swarm<swarm::SubfieldBehaviour>,
		>,
	) -> Result<(), SubfieldError> {
		Ok(())
	}
}
