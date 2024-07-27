use super::*;
use crate::*;
use std::future::Future;

#[derive(Debug)]
pub enum SubfieldError {
	GetRequestFailed,
	PutRequestFailed,
	Config(SubfieldConfigError),
	HandleEvents(SwarmHandleEventError),
	Topic(SubfieldTopicError),
	// Get(GetError),
}

#[derive(Debug)]
pub enum SubfieldConfigError {
	InvalidSwarmConfig,
	InvalidStoreConfig,
}

#[derive(Debug)]
pub enum SwarmHandleEventError {
	FailedToLockSwarm,
	AllEventsFailed,
	SwarmExitedUnexpectedly,
	IncomingMessageQueueClosed,
	OutgoingMessageQueueClosed,
}

pub struct SubfieldConfig {
	pub swarm: swarm::SubfieldSwarmConfig,
	pub store: store::SubfieldStoreConfig,
}

pub trait Subfield {
	fn new(
		config: SubfieldConfig,
	) -> impl Future<Output = Result<Self, SubfieldError>>
	where
		Self: Sized;

	fn event_loop(&self) -> impl Future<Output = Result<(), SubfieldError>>;

	fn put(
		&self,
		author: LocalAuthor,
		topic: SubfieldTopic,
		entry: SubfieldEntry,
	) -> impl Future<Output = Result<(), SubfieldError>>;

	fn get(
		&self,
		topic: SubfieldTopic,
	) -> impl Future<Output = Result<GetResponse, SubfieldError>>;
}
