use crate::*;
use ::core::task::{Context, Poll};
use derive_prelude::PortUse;
use libp2p::core::*;
use libp2p::swarm::handler::ConnectionEvent;
use libp2p::swarm::*;
use libp2p::{InboundUpgrade, OutboundUpgrade};

pub struct ChordConnectionHandler {
	protocol_config: ChordProtocolConfig,
	mode: ChordMode,
}

#[derive(Clone, Debug)]
pub enum ChordFromBehaviourToHandler {}

#[derive(Clone, Debug)]
pub enum ChordFromHandlerToBehaviour {}

impl ConnectionHandler for ChordConnectionHandler {
	// A type representing the message(s) a NetworkBehaviour can send to a ConnectionHandler via ToSwarm::NotifyHandler
	// type FromBehaviour: Debug + Send + 'static;
	type FromBehaviour = ChordFromBehaviourToHandler;
	// A type representing message(s) a ConnectionHandler can send to a NetworkBehaviour via ConnectionHandlerEvent::NotifyBehaviour.
	// type ToBehaviour: Debug + Send + 'static;
	type ToBehaviour = ChordFromHandlerToBehaviour;
	// The inbound upgrade for the protocol(s) used by the handler.
	// type InboundProtocol: InboundUpgradeSend;
	type InboundProtocol = Either<ChordProtocolConfig, upgrade::DeniedUpgrade>;
	// The outbound upgrade for the protocol(s) used by the handler.
	// type OutboundProtocol: OutboundUpgradeSend;
	type OutboundProtocol = ChordProtocolConfig;
	// The type of additional information returned from listen_protocol.
	// type InboundOpenInfo: Send + 'static;
	type InboundOpenInfo = ();
	// The type of additional information passed to an OutboundSubstreamRequest.
	// type OutboundOpenInfo: Send + 'static;
	type OutboundOpenInfo = ();

	// Required methods

	/*
	The InboundUpgrade to apply on inbound substreams to negotiate the desired protocols.

	Note: The returned InboundUpgrade should always accept all the generally supported protocols, even if in a specific context a particular one is not supported, (eg. when only allowing one substream at a time for a protocol). This allows a remote to put the list of supported protocols in a cache.

	*/
	fn listen_protocol(
		&self,
	) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
		match self.mode {
			ChordMode::Server => SubstreamProtocol::new(
				Either::Left(self.protocol_config.clone()),
				(),
			),
			ChordMode::Client => SubstreamProtocol::new(
				Either::Right(upgrade::DeniedUpgrade),
				(),
			),
		}
	}

	fn poll(
		&mut self,
		cx: &mut Context<'_>,
	) -> Poll<
		ConnectionHandlerEvent<
			Self::OutboundProtocol,
			Self::OutboundOpenInfo,
			Self::ToBehaviour,
		>,
	> {
		unimplemented!()
	}

	/*
	Informs the handler about an event from the NetworkBehaviour.
	*/
	fn on_behaviour_event(&mut self, event: Self::FromBehaviour) {
		match event {
			// ChordFromBehaviourToHandler::
			_ => unimplemented!(),
		}
	}

	fn on_connection_event(
		&mut self,
		event: ConnectionEvent<
			'_,
			Self::InboundProtocol,
			Self::OutboundProtocol,
			Self::InboundOpenInfo,
			Self::OutboundOpenInfo,
		>,
	) {
		unimplemented!()
	}
}
