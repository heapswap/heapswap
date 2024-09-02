use crate::*;
use generational_arena::{Arena, Index};
use libp2p::request_response::{InboundRequestId, OutboundRequestId};

pub struct ClientReceiver<T>(Receiver<T>);


/*
 The purpose of the switchboard is to 
 	- keep track of all the subscriptions to keys
	- route messages to connected peers
	- handle peers joining and leaving
	
	
	process
	
	- when a peer connects, add it to the switchboard
	
	
	
	
	
	
	
	
	
*/
#[derive(Debug)]
pub struct Switchboard {
	// streams: DashMap<PeerId, Stream>,

}

impl Switchboard {
	// pub fn new() -> Self {
	// 	Self {
	// 	}
	// }
}


#[async_trait]
pub trait ClientSwitchboard {

}

#[async_trait]
impl ClientSwitchboard for SubfieldClient {
	
	
	
}


pub async fn send<T>(message: T, mut stream: &mut Stream) -> Result<(), SubfieldError>
where
    T: Serialize,
{
	let bytes = cbor_serialize(&message)?;
	
	let len: [u8; 8] = bytes.len().to_le_bytes();
	
	stream.write_all(&len).await.map_err(|e| SubfieldError::FailedToWriteStream)?;
	stream.write_all(&bytes).await.map_err(|e| SubfieldError::FailedToWriteStream)?;
	
	stream.close().await.map_err(|e| SubfieldError::FailedToCloseStream)?;

	Ok(())
}

pub async fn recv<T>(mut stream: &mut Stream) -> Result<T, SubfieldError>
where
    T: for<'de> Deserialize<'de>,
{
	let mut len_buf = [0; 8];
	stream.read_exact(&mut len_buf).await.map_err(|e| SubfieldError::FailedToReadStream)?;
	let len = usize::from_le_bytes(len_buf);

	let mut bytes = vec![0; len];
	stream.read_exact(&mut bytes).await.map_err(|e| SubfieldError::FailedToReadStream)?;
	
	Ok(cbor_deserialize(&bytes)?)
}