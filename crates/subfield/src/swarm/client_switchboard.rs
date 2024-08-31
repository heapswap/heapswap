use crate::*;
use generational_arena::{Arena, Index};
use libp2p::request_response::{InboundRequestId, OutboundRequestId};

pub struct ClientReceiver<T>(Receiver<T>);


#[async_trait]
pub trait ClientSwitchboard {
	
	// fn recv_response_from_swarm(&self, outbound_id: OutboundRequestId, message: SubfieldResponse) -> () {}
	
	// fn create_handoff(&self, outbound_id: OutboundRequestId, inbound_id: InboundRequestId) -> () {}
	
	// fn send_request<T>(&self, request: SubfieldRequest) -> ClientReceiver<T> {
	// 	let (tx, rx) = unbounded();
	// 	ClientReceiver(rx) 
	// }
	
}

#[async_trait]
impl ClientSwitchboard for SubfieldClient {
	
	
	
}