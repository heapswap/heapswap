use crate::*;

#[async_trait]
impl SubfieldHandlerTrait<EchoRequest> for SubfieldClient {
	
	async fn handle_self_is_closest<'a>(
		&self,
		handle: u64,
		request: EchoRequest,
		response_channel: ResponseChannel<SubfieldResponse>,
		swarm: &mut MutexGuard<'a, libp2p::swarm::Swarm<swarm::SubfieldBehaviour>>,
	)-> Result<(), SubfieldError>{
		let response = SubfieldResponse::Echo(Ok(EchoSuccess {
			message: request.message,
		}));
		let _ = swarm.behaviour_mut().subfield.send_response(response_channel, response);
		Ok(())
	}
	
	async fn handle_self_is_not_closest<'a>(
		&self,
		handle: u64,
		request: EchoRequest,
		response_channel: ResponseChannel<SubfieldResponse>,
		closest_peer: libp2p::PeerId,
		swarm: &mut MutexGuard<'a, libp2p::swarm::Swarm<swarm::SubfieldBehaviour>>,
	)-> Result<(), SubfieldError>{
		self.handle_self_is_closest(handle, request, response_channel, swarm).await
	}
	
}