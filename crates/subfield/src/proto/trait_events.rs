use crate::*;


#[async_trait]
pub trait SubfieldEventsTrait {
	
	/**
	 * Bootstrap
	*/
	async fn bootstrap(&self) -> Result<(), SubfieldError>;
	
	/**
	 * Event loop
	*/
	async fn event_loop(&self);
	
	/**
	 * Peer selection
	*/
	async fn random_local_peer(&self) -> Option<libp2p::PeerId>;
	
	async fn closest_local_peer(&self, subkey: RoutingSubkey) -> Result<Option<libp2p::PeerId>, SubfieldError>;
	
	
	/**
	 * Send requests
	*/
	async fn send_request(&self, peer_id: libp2p::PeerId, request: SubfieldRequest) -> Result<u64, SubfieldError>;
	
	async fn send_request_to_closest_local_peer(&self, subkey: RoutingSubkey, request: SubfieldRequest) -> Result<u64, SubfieldError>;
	
}