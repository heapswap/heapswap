use crate::*;

use libp2p_stream::OpenStreamError;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, strum::Display, Error)]
pub enum SubfieldError {
	NoGlobalPeers,
	BootstrapFailedNoMultiaddrs,
	BootstrapFailedNoUrls,
	BootstrapFailedDial,
	IncompleteKey,
	CompleteKeyMissingField,
	RoutingKeyMissingField,
	NoConnectedPeers,
	NoLocalPeer,
	SelfIsClosest,
	RequestTimeout,
	RequestFailed,
	UnexpectedResponseType,
	PortalError(PortalError),
	ChannelClosed,
	SwarmError,
	FailedToOpenStream,
	FailedToWriteStream,
	FailedToReadStream,
	FailedToCloseStream,
}
