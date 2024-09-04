use crate::*;
use libp2p::StreamProtocol;

/// The protocol name used for negotiating with multistream-select.
pub(crate) const DEFAULT_PROTO_NAME: StreamProtocol =
	StreamProtocol::new("/subfield/chord/1.0.0");
/// The default maximum size for a varint length-delimited packet.
pub(crate) const DEFAULT_MAX_PACKET_SIZE: usize = 16 * 1024;
