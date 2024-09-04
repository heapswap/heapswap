use crate::*;
use ::core::task::{Context, Poll};
use derive_prelude::PortUse;
use libp2p::core::*;
use libp2p::swarm::*;

pub enum ChordMode {
	Server,
	Client,
}

pub struct ChordBehaviourConfig {
	pub mode: ChordMode,
}

impl Default for ChordBehaviourConfig {
	fn default() -> Self {
		ChordBehaviourConfig {
			mode: ChordMode::Client,
		}
	}
}
