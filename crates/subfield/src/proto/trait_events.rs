use libp2p::request_response::OutboundRequestId;

use crate::*;


/*
	The events trait is the internal trait for handling events
*/

#[async_trait]
pub trait SubfieldEventsTrait {
	/*
	 Bootstrap
	*/
	async fn bootstrap(&self) -> Result<(), SubfieldError>;

	/*
	 Event loop
	*/
	async fn event_loop(&self);

	/*
	 Peer selection
	*/

	async fn closest_local_peer(
		&self,
		key: &RoutingKey,
	) -> Result<Option<libp2p::PeerId>, SubfieldError>;

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
