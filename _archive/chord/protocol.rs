// use crate::proto;
// use crate::record::{self, Record};
use asynchronous_codec::{Decoder, Encoder, Framed};
// use bytes::BytesMut;
use futures::prelude::*;
use libp2p::core::upgrade::{InboundUpgrade, OutboundUpgrade, UpgradeInfo};

use std::marker::PhantomData;
use std::time::Duration;
use std::{io, iter};
// use tracing::debug;
// use web_time::Instant;
use crate::*;
use libp2p::StreamProtocol;

/// Status of our connection to a node reported by the Chord protocol.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ConnectionType {
	/// Sender hasn't tried to connect to peer.
	NotConnected = 0,
	/// Sender is currently connected to peer.
	Connected = 1,
	/// Sender was recently connected to peer.
	CanConnect = 2,
	/// Sender tried to connect to peer but failed.
	CannotConnect = 3,
}

/// Information about a peer, as known by the sender.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChordPeer {
	/// Identifier of the peer.
	pub node_id: PeerId,
	/// The multiaddresses that the sender think can be used in order to reach the peer.
	pub multiaddrs: Vec<Multiaddr>,
	/// How the sender is connected to that remote.
	pub connection_ty: ConnectionType,
}

/// Configuration for a Chord connection upgrade. When applied to a connection, turns this
/// connection into a `Stream + Sink` whose items are of type `ChordRequestMsg` and `ChordResponseMsg`.
#[derive(Debug, Clone)]
pub struct ChordProtocolConfig {
	protocol_names: Vec<StreamProtocol>,
	/// Maximum allowed size of a packet.
	max_packet_size: usize,
}

impl ChordProtocolConfig {
	/// Builds a new `ChordProtocolConfig` with the given protocol name.
	pub fn new(protocol_name: StreamProtocol) -> Self {
		ChordProtocolConfig {
			protocol_names: vec![protocol_name],
			max_packet_size: DEFAULT_MAX_PACKET_SIZE,
		}
	}

	/// Returns the default configuration.
	#[deprecated(note = "Use `ChordProtocolConfig::new` instead")]
	#[allow(clippy::should_implement_trait)]
	pub fn default() -> Self {
		Default::default()
	}

	/// Returns the configured protocol name.
	pub fn protocol_names(&self) -> &[StreamProtocol] {
		&self.protocol_names
	}

	/// Modifies the protocol names used on the wire. Can be used to create incompatibilities
	/// between networks on purpose.
	#[deprecated(note = "Use `ChordProtocolConfig::new` instead")]
	pub fn set_protocol_names(&mut self, names: Vec<StreamProtocol>) {
		self.protocol_names = names;
	}

	/// Modifies the maximum allowed size of a single Chord packet.
	pub fn set_max_packet_size(&mut self, size: usize) {
		self.max_packet_size = size;
	}
}

impl Default for ChordProtocolConfig {
	/// Returns the default configuration.
	///
	/// Deprecated: use `ChordProtocolConfig::new` instead.
	fn default() -> Self {
		ChordProtocolConfig {
			protocol_names: iter::once(DEFAULT_PROTO_NAME).collect(),
			max_packet_size: DEFAULT_MAX_PACKET_SIZE,
		}
	}
}

impl UpgradeInfo for ChordProtocolConfig {
	type Info = StreamProtocol;
	type InfoIter = std::vec::IntoIter<Self::Info>;

	fn protocol_info(&self) -> Self::InfoIter {
		self.protocol_names.clone().into_iter()
	}
}

/// Sink of responses and stream of requests.
pub(crate) type ChordInStreamSink<S> =
	Framed<S, Codec<ChordResponseMsg, ChordRequestMsg>>;
/// Sink of requests and stream of responses.
pub(crate) type ChordOutStreamSink<S> =
	Framed<S, Codec<ChordRequestMsg, ChordResponseMsg>>;

impl<C> InboundUpgrade<C> for ChordProtocolConfig
where
	C: AsyncRead + AsyncWrite + Unpin,
{
	type Output = ChordInStreamSink<C>;
	type Future = future::Ready<Result<Self::Output, io::Error>>;
	type Error = io::Error;

	fn upgrade_inbound(self, incoming: C, _: Self::Info) -> Self::Future {
		let codec = Codec::new(self.max_packet_size);

		future::ok(Framed::new(incoming, codec))
	}
}

impl<C> OutboundUpgrade<C> for ChordProtocolConfig
where
	C: AsyncRead + AsyncWrite + Unpin,
{
	type Output = ChordOutStreamSink<C>;
	type Future = future::Ready<Result<Self::Output, io::Error>>;
	type Error = io::Error;

	fn upgrade_outbound(self, incoming: C, _: Self::Info) -> Self::Future {
		let codec = Codec::new(self.max_packet_size);

		future::ok(Framed::new(incoming, codec))
	}
}

/// Request that we can send to a peer or that we received from a peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChordRequestMsg {
	/// Ping request.
	Ping,
}

/// Response that we can send to a peer or that we received from a peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChordResponseMsg {
	/// Ping response.
	Pong,
}
