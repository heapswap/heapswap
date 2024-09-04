use crate::*;

/*
 This is the main handler for requests. It is responsible for dispatching to the correct handler based on the body type.
*/

#[async_trait]
impl SubfieldDispatchTrait for SubfieldClient {
	async fn dispatch_request<'a>(
		&self,
		request: SubfieldRequest,
		swarm: &mut MutexGuard<
			'a,
			libp2p::swarm::Swarm<swarm::SubfieldNetworkBehaviour>,
		>,
	) -> Result<(), SubfieldError> {
		let routing_key = request.routing_key;
		match request.body {
			SubfieldRequestBody::Echo(body) => {
				self.handle_request(routing_key, body, swarm).await?;
			}
			_ => {}
		}
		Ok(())
	}
}
