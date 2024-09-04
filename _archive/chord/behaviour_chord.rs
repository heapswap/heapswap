use crate::*;
use ::core::task::{Context, Poll};
use derive_prelude::PortUse;
use libp2p::core::*;
use libp2p::swarm::*;

pub struct ChordBehaviour {
	config: ChordBehaviourConfig,
}

impl ChordBehaviour {
	pub fn new(config: ChordBehaviourConfig) -> Self {
		Self { config }
	}

	pub fn set_mode(&mut self, mode: ChordMode) {
		self.config.mode = mode
	}
}
