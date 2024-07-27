use std::pin::Pin;

use super::*;
use crate::*;
use futures::future::{self, FutureExt};
use futures::prelude::*;
#[cfg(feature = "browser")]
use gloo::timers::future::TimeoutFuture;
#[cfg(feature = "server")]
use libp2p::mdns;
use libp2p::{gossipsub, kad, ping, swarm::SwarmEvent, Swarm};

#[derive(Debug)]
pub enum SwarmHandleEventError {
	FailedToLockSwarm,
	AllEventsFailed,
	SwarmExitedUnexpectedly,
	IncomingMessageQueueClosed,
	OutgoingMessageQueueClosed,
}

enum EventOutcome {
	SubfieldSwarmEvent(SwarmEvent<swarm::behaviour::SubfieldBehaviourEvent>),
	SubfieldMessage(SubfieldMessage),
}

pub async fn handle_events(
	mut store: &mut store::SubfieldStore,
	mut swarm: &mut SubfieldSwarm,
	mut rx: &mut Receiver<SubfieldMessage>,
	mut tx: &mut Transmitter<SubfieldMessage>,
) -> Result<(), SwarmHandleEventError> {
	let swarm_future = swarm
		.next()
		.map(|event| Ok(event.map(EventOutcome::SubfieldSwarmEvent)));
	let rx_future = rx
		.next()
		.map(|request| Ok(request.map(EventOutcome::SubfieldMessage)));

	let res = future::select_ok([
		Box::pin(swarm_future)
			as Pin<
				Box<
					dyn Future<
							Output = Result<
								Option<EventOutcome>,
								SwarmHandleEventError,
							>,
						> + Send,
				>,
			>,
		Box::pin(rx_future)
			as Pin<
				Box<
					dyn Future<
							Output = Result<
								Option<EventOutcome>,
								SwarmHandleEventError,
							>,
						> + Send,
				>,
			>,
	])
	.await;

	let (event_outcome, _) =
		res.map_err(|_| SwarmHandleEventError::AllEventsFailed)?;

	match event_outcome {
		Some(EventOutcome::SubfieldSwarmEvent(event)) => {
			handle_swarm_event(&mut store, &mut swarm, &mut tx, event).await?;
		}
		Some(EventOutcome::SubfieldMessage(message)) => {
			handle_subfield_message(&mut store, &mut swarm, &mut tx, message).await?;
		}
		None => {
			return Err(SwarmHandleEventError::SwarmExitedUnexpectedly);
		}
	}

	Ok(())
}

/*
match res {
	Ok((event_outcome, _)) => {
		match event_outcome {
			Some(EventOutcome::SubfieldSwarmEvent(event)) => {
				handle_swarm_event(&mut swarm, &mut tx, event).await?;
			}
			Some(EventOutcome::SubfieldMessage(message)) => {
				handle_subfield_message(&mut swarm, &mut tx, message).await?;
			}
			None => {
				return Err(SwarmHandleEventError::SwarmExitedUnexpectedly);
			}
		}
	}
	Err(_) => {
		return Err(SwarmHandleEventError::AllEventsFailed);
	}
}
*/

/*
// Check the swarm event queue for messages
async fn handle_next_swarm_event(
	mut swarm: &mut SubfieldSwarm,
	mut tx: &mut Transmitter<SubfieldMessage>,
) -> Result<(), SwarmHandleEventError> {
	match swarm.next().await {
		Some(event) => {
			let _ = swarm::handle_swarm_event(&mut swarm, &mut tx, event).await;
		}
		None => {
			tracing::error!("Swarm exited unexpectedly");
			return Err(SwarmHandleEventError::SwarmExitedUnexpectedly);
		}

	}
	Ok(())
}
*/

/*
let mut tx_clone = tx.clone();
let events = vec![
	handle_next_swarm_event(&mut swarm, &mut tx_clone).boxed(),
	handle_next_subfield_message(&mut rx, &mut tx).boxed(),
];

future::select_ok(events)
	.await
	.map(|_| ())
	.map_err(|e| SwarmHandleEventError::AllEventsFailed)
*/

/*
// check the incoming queue for messages to process
async fn handle_next_subfield_message(
	mut rx: &mut Receiver<SubfieldResponse>,
	mut tx: &mut Transmitter<SubfieldMessage>,
) -> Result<(), SwarmHandleEventError> {
	match rx.next().await {
		Some(message) => {
			let _ = swarm::handle_subfield_message(&mut tx, message).await;
		}
		None => {
			tracing::error!("Incoming message queue is closed");
		}
	}
	Ok(())
}
*/

/*
pub struct HandleEventOptions {
	pub swarm: SubfieldSwarm,
	// incoming messages
	pub incoming: Portal<SubfieldResponse, SubfieldMessage>,
	// outgoing messages
	pub outgoing: Portal<SubfieldMessage, SubfieldResponse>,
}

pub struct HandleSwarmEventOptions {
	pub swarm: SubfieldSwarm,
	pub incoming_tx: Transmitter<SubfieldMessage>,
	pub outgoing_tx: Transmitter<SubfieldResponse>,
}

pub struct HandleIncomingMessageOptions {
	pub incoming_tx: Transmitter<SubfieldResponse>,
	pub incoming_rx: Receiver<SubfieldMessage>,
	pub outgoing_tx: Transmitter<SubfieldResponse>,
}

pub struct HandleOutgoingMessageOptions {
	pub incoming_tx: Transmitter<SubfieldMessage>,
	pub outgoing_tx: Transmitter<SubfieldResponse>,
	pub outgoing_rx: Receiver<SubfieldMessage>,
}

#[derive(Clone)]
pub struct Transmitters {
	pub incoming: Transmitter<SubfieldResponse>,
	pub outgoing: Transmitter<SubfieldMessage>,
}

pub struct MultiplexedSwarm {
	pub transmitters: Transmitters,
	pub swarm_opt: HandleSwarmEventOptions,
	pub incoming_opt: HandleIncomingMessageOptions,
	pub outgoing_opt: HandleOutgoingMessageOptions,
}

impl MultiplexedSwarm {
	pub fn new(swarm: SubfieldSwarm) -> Self {


		let (incoming_tx, incoming_rx) = swarm::subfield_incoming_channel();
		let (outgoing_tx, outgoing_rx) = swarm::subfield_outgoing_channel();

		Self {
			transmitters: Transmitters {
				incoming: incoming_tx.clone(),
				outgoing: outgoing_tx.clone(),
			},
			swarm_opt: HandleSwarmEventOptions {
					swarm: swarm,
					incoming_tx: incoming_tx.clone(),
					outgoing_tx: outgoing_tx.clone(),
				},
				incoming_opt: HandleIncomingMessageOptions {
					incoming_tx: incoming_tx.clone(),
					outgoing_tx: outgoing_tx.clone(),
					incoming_rx: incoming_rx,
				},
				outgoing_opt: HandleOutgoingMessageOptions {
					incoming_tx: incoming_tx.clone(),
					outgoing_tx: outgoing_tx.clone(),
					outgoing_rx: outgoing_rx,
				},
		}
	}

	pub fn tx(&self) -> Transmitters {
		self.transmitters.clone()
	}
}
*/

// pub struct HandleEventOptions {
// 	pub swarm: SubfieldSwarm,
// 	pub tx: Transmitter<SubfieldMessage>,
// 	pub rx: Receiver<SubfieldMessage>,
// }

/*
// Check the incoming queue for messages to process
pub async fn handle_next_incoming_message(mut opt: &mut HandleIncomingMessageOptions) -> Result<(), SwarmHandleEventError> {
	match opt.incoming_rx.next().await{
		Some(message) => {
			let _ = swarm::handle_incoming_message(&mut opt, message).await;
		}
		None => {
			tracing::error!("Incoming message queue is closed");
		}
	}
	Ok(())
}


// Check the outgoing queue for messages to process and send
pub async fn handle_next_outgoing_message(mut opt: &mut HandleOutgoingMessageOptions) -> Result<(), SwarmHandleEventError> {
	match opt.outgoing_rx.next().await {
		Some(message) => {
				let _ = handle_outgoing_message(&mut opt, message).await;
			}
		None => {
			tracing::error!("Outgoing message queue is closed");
			return Err(SwarmHandleEventError::OutgoingMessageQueueClosed);
		}
	}
	Ok(())
}
*/

/*
// This function takes ownership of all of the arguments, as the swarm is meant to be interacted with via the tx and rx channels only
pub async fn handle_events(mut opt: HandleEventOptions) {

		// handle swarm event
		let _ = handle_next_swarm_event(&mut opt).await;

		// handle incoming message
		let _ = handle_next_incoming_message(&mut opt).await;

		// handle outgoing message
		let _ = handle_next_outgoing_message(&mut opt).await;

		// empty promise to yield event loop
		#[cfg(feature = "browser")]
		TimeoutFuture::new(0).await;
}



// Things to note about these functions:
//  - Despite being async, they try to avoid blocking the thread, yielding immediately
//     - This is because there isnt a tokio::select for the browser
//  - They take channels in order to recursively call themselves, unsure if this will be used



// Check the incoming swarm queue for messages
pub async fn handle_next_swarm_event(mut opt: &mut HandleEventOptions) {
	match opt.swarm.next().now_or_never() {
		Some(event) => match event {
			Some(event) => {
				let _ = swarm::handle_swarm_event(&mut opt, event).await;
			}
			None => {
				tracing::error!("Swarm exited unexpectedly");
			}
		},
		_ => {}
	}
}


// Check the outgoing local queue for messages
pub async fn handle_next_incoming_message_(mut opt: &mut HandleEventOptions) {

	match opt.incoming_rx.next().now_or_never() {
		Some(message) => match message {
			Some(message) => {
				let _ = swarm::handle_outgoing_message(&mut opt, message).await;
			}
			None => {
				tracing::error!("Swarm exited unexpectedly");
			}
		},
		_ => {}
	}
}

// Check the outgoing local queue for messages
pub async fn handle_next_outgoing_message(mut opt: &mut HandleEventOptions) {

	match opt.outgoing_rx.next().now_or_never() {
		Some(message) => match message {
			Some(message) => {
				let _ = handle_outgoing_message(&mut opt, message).await;
			}
			None => {
				tracing::error!("Swarm exited unexpectedly");
			}
		},
		_ => {}
	}
}
*/
