use crate::*;

#[async_trait]
impl SubfieldServiceTrait for SubfieldClient {
	async fn echo(&self, key: CompleteKey, message: &str) -> Result<String, SubfieldError> {
		
		let routing_key = key.to_signer_routing_key();
		
		let closest_peer = self.closest_local_peer(&routing_key).await?;
		
		if closest_peer.is_none() {
			return Err(SubfieldError::SelfIsClosest);
		}
		let closest_peer = closest_peer.unwrap();
		
		let mut s = self.new_stream(closest_peer).await?;
		
		send(EchoRequest { message: message.to_string() }, &mut s).await?;
			
		tracing::info!("echoed message to peer {}", closest_peer);
		
		let response = recv::<EchoResponse>(&mut s).await?.map_err(|e| SubfieldError::EchoFailure)?;
		
		Ok(response.message)
	}
}
