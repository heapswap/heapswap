use super::*;
use crate::*;
use dashmap::DashMap;
use futures::channel::{mpsc, oneshot};
use futures::future::{self, FutureExt};
use futures::join;
use futures::prelude::*;
use getset::{Getters, Setters};
#[cfg(feature = "browser")]
use gloo::timers::future::TimeoutFuture;
use libp2p::kad;
use libp2p::request_response::OutboundRequestId;
use std::pin::Pin;

pub type RequestCallback = Box<dyn FnOnce()>;
pub type SubscriptionCallback = Box<dyn FnOnce()>;

#[derive(Getters, Setters)]
pub struct SubfieldClient {
	// the libp2p swarm handles networking
	swarm: Mutex<swarm::SubfieldSwarm>,
	// incoming_tx and rx help insert things into the swarm event loop
	incoming_tx: Transmitter<SubfieldRequest>,
	incoming_rx: Mutex<Receiver<SubfieldRequest>>,

	requests: DashMap<OutboundRequestId, oneshot::Sender<SubfieldResponse>>,
	subscriptions: DashMap<OutboundRequestId, mpsc::Sender<SubfieldResponse>>,

	// the store handles local state
	store: store::SubfieldStore,
}


impl SubfieldClient{
	
pub async fn handle_subfield_request(
&mut self,
mut swarm: &mut swarm::SubfieldSwarm,
request: SubfieldRequest,
) -> Result<(), SwarmHandleEventError> {
	match request {
		SubfieldRequest::Put(signed_put_request) => {
			
			let put_request = signed_put_request.put_request;
			
			// verify that the topic has all three fields
			let _ = put_request.topic_full.is_complete().map_err(|e| SubfieldError::Topic(e));


			let mut receivers: Vec<oneshot::Receiver<SubfieldResponse>> =
				Vec::with_capacity(3);

			for (current_topic_part, current_topic) in put_request.topic_full.parts() {
				// puts always have all three parts
				let current_topic = current_topic.unwrap();

				// build put subrequest
				let unsigned_put_request = PutRequest {
					author: put_request.author,
					topic_part: current_topic_part,
					topic_full: put_request.topic_full.clone(),
					entry: put_request.entry.clone(),
				};

				// find the closest local node to the subtopic
				let key = kad::KBucketKey::from(current_topic.to_bytes());
				let closest_local_peer = swarm
					.behaviour_mut()
					.kademlia
					.get_closest_local_peers(&key)
					.next()
					.unwrap();
				let closest_local_peer = closest_local_peer.preimage();

				// create and sign the request
				let unsigned_message_bytes =
					bincode::serialize(&unsigned_put_request).unwrap();
				let put_request = SubfieldRequest::Put(PutRequest {
					unsigned_request: unsigned_put_request,
					author_data_signature: author
						.keypair()
						.sign(&unsigned_message_bytes),
				});

				// send the request to the closest local node
				let request_id = swarm
					.behaviour_mut()
					.subfield
					.send_request(closest_local_peer, put_request);

				let (tx, rx) = oneshot::channel::<SubfieldResponse>();

				// add the request to the requests map
				self.requests.insert(request_id, tx);
				receivers.push(rx);
			}

			// wait for all the responses
			let results: Vec<Result<SubfieldResponse, oneshot::Canceled>> =
				futures::future::join_all(receivers).await;

			for result in results {
				if result.is_err() {
					return Err(SubfieldError::PutRequestFailed);
				}
			}
		}
		SubfieldRequest::Get(get_request) => {
			// gets require all three parts
			let _ = topic.is_complete().map_err(|e| SubfieldError::Topic(e));

			// get the swarm
			let mut swarm = self.swarm.lock().await;

			// create the request channel
			let (tx, rx) = oneshot::channel::<SubfieldResponse>();

			let mut receivers: Vec<oneshot::Receiver<SubfieldResponse>> =
				Vec::with_capacity(3);

			// find the nearest peer
			let mut request_id: OutboundRequestId;
			for (current_topic_part, current_topic) in topic.parts() {
				let current_topic = current_topic.unwrap();

				// build the get request
				let get_request = SubfieldRequest::Get(GetRequest {
					topic_part: current_topic_part,
					topic_full: topic.clone(),
				});

				// find the closest local node to the subtopic
				let key = kad::KBucketKey::from(current_topic.to_bytes());
				let closest_local_peer = swarm
					.behaviour_mut()
					.kademlia
					.get_closest_local_peers(&key)
					.next()
					.unwrap();
				let closest_local_peer = closest_local_peer.preimage();

				// send the request
				let request_id = swarm
					.behaviour_mut()
					.subfield
					.send_request(closest_local_peer, get_request);

				// create the receiver
				let (tx, rx) = oneshot::channel::<SubfieldResponse>();

				// add the request to the requests map
				self.requests.insert(request_id, tx);
				receivers.push(rx);
			}

			// Use select_ok to get the first successful response
			let (response, _) = futures::future::select_ok(receivers)
				.await
				.map_err(|_| SubfieldError::GetRequestFailed)?;

			let response = match response {
				SubfieldResponse::Get(get_response) => get_response,
				_ => return Err(SubfieldError::GetRequestFailed),
			};
		}
	}
	Ok(())
}

}

enum EventOutcome {
	SubfieldSwarmEvent(
		libp2p::swarm::SwarmEvent<swarm::SubfieldBehaviourEvent>,
	),
	SubfieldRequest(SubfieldRequest),
}

impl Subfield for SubfieldClient {
	async fn new(config: SubfieldConfig) -> Result<Self, SubfieldError> {
		let swarm: Mutex<swarm::SubfieldSwarm> =
			Mutex::new(swarm::create(config.swarm).await.map_err(|_| {
				SubfieldError::Config(SubfieldConfigError::InvalidSwarmConfig)
			})?);

		let (incoming_tx, incoming_rx) = portal::<SubfieldRequest>();
		let incoming_rx = Mutex::new(incoming_rx);

		let store = store::create(config.store).await.map_err(|_| {
			SubfieldError::Config(SubfieldConfigError::InvalidStoreConfig)
		})?;

		let requests = DashMap::new();
		let subscriptions = DashMap::new();

		Ok(Self {
			swarm,
			incoming_tx,
			incoming_rx,
			requests,
			subscriptions,
			store,
		})
	}

	async fn event_loop(&self) -> Result<(), SubfieldError> {
		/*
		loop {

			// check swarm
			let mut lock_swarm = self.swarm.lock().await;

			match lock_swarm.next().now_or_never() {
				Some(event) => {
					match event {
						Some(event) => {
							// handle_swarm_event(&mut store, &mut swarm, &mut tx, event).await?;
						}
						None => {}
					}
				},
				None => {
					return Err(SubfieldError::HandleEvents(
						SwarmHandleEventError::SwarmExitedUnexpectedly,
					));
				}
			}

			// empty promise to yield event loop
			#[cfg(feature = "browser")]
			let _ = TimeoutFuture::new(0).await;
			#[cfg(feature= "server")]
			let _ = tokio::task::yield_now().await;
		}
		*/

		let mut swarm_lock = self.swarm.lock().await;

		let mut incoming_rx = self.incoming_rx.lock().await;

		loop {
			let swarm_future = swarm_lock
				.next()
				.map(|event| Ok(event.map(EventOutcome::SubfieldSwarmEvent)));
			let rx_future = incoming_rx
				.next()
				.map(|request| Ok(request.map(EventOutcome::SubfieldRequest)));

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

			let (event_outcome, _) = res.map_err(|_| {
				SubfieldError::HandleEvents(
					SwarmHandleEventError::AllEventsFailed,
				)
			})?;

			match event_outcome {
				Some(EventOutcome::SubfieldSwarmEvent(event)) => {
					// handle_swarm_event(&mut store, &mut swarm, &mut tx, event).await?;
				}
				Some(EventOutcome::SubfieldRequest(request)) => {
					// handle_subfield_message(&mut store, &mut swarm, &mut tx, message).await?;
				}
				None => {
					return Err(SubfieldError::HandleEvents(
						SwarmHandleEventError::SwarmExitedUnexpectedly,
					));
				}
			}
		}
	}

	async fn put(
		&self,
		author: LocalAuthor,
		topic: SubfieldTopic,
		entry: SubfieldEntry,
	) -> Result<(), SubfieldError> {
		Ok(())
	}

	async fn get(
		&self,
		topic: SubfieldTopic,
	) -> Result<GetResponse, SubfieldError> {
		todo!()
		// Ok(response)
	}
}

/*

*/
