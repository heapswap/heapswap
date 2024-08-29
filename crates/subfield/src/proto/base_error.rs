use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubfieldError {
	IncompleteSubkey,
	NoConnectedPeers,
	NoLocalPeer,
	SelfIsClosest,
	RequestTimeout,
	RequestFailed,
	UnexpectedResponseType,
	PortalError(PortalError),
	ChannelClosed
}