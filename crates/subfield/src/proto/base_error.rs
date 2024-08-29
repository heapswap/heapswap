use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubfieldError {
	BootstrapFailedNoMultiaddrs,
	BootstrapFailedNoUrls,
	BootstrapFailedDial,
	IncompleteSubkey,
	CompleteSubkeyMissingField,
	RoutingSubkeyMissingField,
	NoConnectedPeers,
	NoLocalPeer,
	SelfIsClosest,
	RequestTimeout,
	RequestFailed,
	UnexpectedResponseType,
	PortalError(PortalError),
	ChannelClosed,
	SwarmError,
}