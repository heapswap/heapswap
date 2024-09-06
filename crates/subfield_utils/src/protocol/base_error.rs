use crate::*;

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
	ChannelClosed,
	SwarmError,
	FailedToOpenStream,
	FailedToWriteStream,
	FailedToReadStream,
	FailedToCloseStream,
	SerializationFailed,
	DeserializationFailed,
	EchoFailure,
}
