use crate::*;

#[async_trait]
impl SubfieldHandlerTrait<EchoRequest> for SubfieldClient {
	async fn handle_request<'a>(
		&self,
		request_id: InboundRequestId,
		request: EchoRequest,
		response_channel: ResponseChannel<SubfieldResponse>,
		swarm: &mut MutexGuard<
			'a,
			libp2p::swarm::Swarm<swarm::SubfieldBehaviour>,
		>,
	) -> Result<(), SubfieldError> {
		let response = SubfieldResponse::Echo(Ok(EchoSuccess {
			message: request.message,
		}));
		let _ = swarm
			.behaviour_mut()
			.subfield
			.send_response(response_channel, response);
		Ok(())
	}
}
