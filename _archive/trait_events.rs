use libp2p::request_response::OutboundRequestId;

use crate::*;

/*
	The events 
*/

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubfieldMode {
	Client,
	Server,
}


#[async_trait]
pub trait SubfieldEventsTrait {
	/*
	 Bootstrap
	*/
	async fn bootstrap(&self) -> Result<(), SubfieldError>;

	fn set_mode(&mut self, mode: SubfieldMode);

	/*
	 Event loop
	*/
	async fn event_loop(&self);

	/*
	 Peer selection
	*/

	// async fn closest_local_peer(
	// 	&self,
	// 	key: &RoutingKey,
	// ) -> Result<Option<libp2p::PeerId>, SubfieldError>;

	/*
	 Send requests
	*/
	/*
	async fn send_request_to_local_peer(
		&self,
		peer_id: libp2p::PeerId,
		request: SubfieldRequest,
	) -> Result<OutboundRequestId, SubfieldError>;

	async fn send_request_to_closest_local_peer(
		&self,
		request: SubfieldRequest,
	) -> Result<OutboundRequestId, SubfieldError>;
	*/
}
