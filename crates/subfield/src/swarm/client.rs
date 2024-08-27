use libp2p::request_response::{InboundRequestId, OutboundRequestId};

use crate::proto::subfield_response::ResponseType;
pub use crate::*;
use futures::{TryFuture, TryFutureExt};

#[derive(Clone, Debug)]
pub struct SubfieldConfig {
	pub keypair: crypto::Keypair,

	pub mode: SubfieldSwarmMode,

	pub bootstrap_urls: Vec<String>,
	// pub bootstrap_multiaddrs: Vec<String>,
	#[cfg(feature = "server")]
	pub listen_addresses: Vec<String>,

	pub store_path: String,
}

impl Default for SubfieldConfig {
	fn default() -> Self {
		Self {
			keypair: crypto::Keypair::random(),
			mode: SubfieldSwarmMode::Client,
			bootstrap_urls: vec![
				"localhost:3000/bootstrap".to_string(),
				"heapswap.com/bootstrap".to_string(),
			],
			#[cfg(feature = "server")]
			listen_addresses: vec![
				// "/ip4/0.0.0.0/tcp/3000/webrtc".to_string(),
				// "/ip6/::/tcp/3000/webrtc".to_string(),
			],
			store_path: String::from("_subfield_store"),
		}
	}
}

trait SizedMessage: proto::Message + Sized {}

#[derive(Clone)]
pub struct SubfieldClient {
	config: SubfieldConfig,

	swarm: swarm::ThreadsafeSubfieldSwarm,
	store: store::SubfieldStore,

	request_handles: Arc<SubfieldRequestHandles>,
}

type HandleMap<T> = DashMap<OutboundRequestId, Sender<T>>;

enum SubfieldRequestRPC {
	Ping(proto::PingRequest),
	Echo(proto::EchoRequest),
	GetRecord(proto::GetRecordRequest),
	DeleteRecord(proto::DeleteRecordRequest),
	PutRecord(proto::PutRecordRequest),
	Subscribe(proto::SubscribeRequest),
	Unsubscribe(proto::UnsubscribeRequest),
}

enum SubfieldResponseRPC {
	Ping(proto::PingResponse),
	Echo(proto::EchoResponse),
	GetRecord(proto::GetRecordResponse),
	DeleteRecord(proto::DeleteRecordResponse),
	PutRecord(proto::PutRecordResponse),
	Subscribe(Receiver<proto::SubscribeResponse>),
	Unsubscribe(proto::UnsubscribeResponse),
}

/*
struct SubfieldRequestHandles {
	// kad
	kad: DashMap<libp2p::kad::QueryId, Sender<libp2p::PeerId>>,
	// system
	ping: HandleMap<proto::PingResponse>,
	echo: HandleMap<proto::EchoResponse>,
	// records
	get_record: HandleMap<proto::GetRecordResponse>,
	delete_record: HandleMap<proto::DeleteRecordResponse>,
	put_record: HandleMap<proto::PutRecordResponse>,
	// pubsub
	subscribe: HandleMap<proto::SubscribeResponse>,
	unsubscribe: HandleMap<proto::UnsubscribeResponse>,
}

impl SubfieldRequestHandles {
	pub fn new() -> Self {
		Self {
			kad: DashMap::new(),
			ping: HandleMap::new(),
			echo: HandleMap::new(),
			get_record: HandleMap::new(),
			delete_record: HandleMap::new(),
			put_record: HandleMap::new(),
			subscribe: HandleMap::new(),
			unsubscribe: HandleMap::new(),
		}
	}
}
*/

struct SubfieldRequestHandles {
	// pub kad: PortalManager<libp2p::PeerId>,
	pub subfield: PortalManager<SubfieldResponse>,
}

impl SubfieldRequestHandles {
	pub fn new() -> Self {
		Self {
			// kad: PortalManager::new(),
			subfield: PortalManager::new(),
		}
	}
}

fn get_outbound_request_id(request_id: OutboundRequestId) -> u64 {
	unsafe { std::mem::transmute::<OutboundRequestId, u64>(request_id) }
}

fn get_inbound_request_id(request_id: InboundRequestId) -> u64 {
	unsafe { std::mem::transmute::<InboundRequestId, u64>(request_id) }
}

impl SubfieldClient {
	/**
	 * Constructor
		*/
	pub async fn new(config: SubfieldConfig) -> EResult<Self> {
		let swarm: ThreadsafeSubfieldSwarm =
			Arc::new(Mutex::new(swarm::create_swarm(config.clone()).await?));
		let request_handles: Arc<SubfieldRequestHandles> =
			Arc::new(SubfieldRequestHandles::new());
		let store = store::SubfieldStore::new(config.clone()).await?;

		Ok(Self {
			config,
			swarm,
			request_handles,
			store,
		})
	}

	/**
	 * Getters
		*/
	pub async fn swarm(&self) -> MutexGuard<swarm::SubfieldSwarm> {
		self.swarm.lock().await
	}

	pub async fn cache(&self) -> MutexGuard<dyn store::FullStore> {
		self.store.cache().await
	}

	pub async fn perma(&self) -> MutexGuard<dyn store::FullStore> {
		self.store.perma().await
	}

	pub async fn event_loop(&self) {
		loop{
		let mut swarm_lock = self.swarm.lock().await;
		let swarm = &mut *swarm_lock;

		while let Some(Some(event)) = swarm_lock.next().now_or_never() {
			match event {
				swarm::SubfieldSwarmEvent::Behaviour(event) => {
					match event {
						swarm::SubfieldBehaviourEvent::Subfield(event) => {
							match event {
								// an incoming response
								libp2p::request_response::Event::Message {
									peer,
									message,
								} => {
									match message {
										libp2p::request_response::Message::Response { request_id, response } => {
											let handle = get_outbound_request_id(request_id);	
											match response.proto().response_type {
											 	Some(proto::subfield_response::ResponseType::Ping(_)) => {
													let _ = self.request_handles.subfield.send_oneshot(handle, response);										
												},
												Some(proto::subfield_response::ResponseType::Echo(_)) => {
													let _ = self.request_handles.subfield.send_oneshot(handle, response);										
												},
												_ => {}					
											 }
										},	
										_ => {}
									}
								}
								_ => {}
							}
						}
						/*
						swarm::SubfieldBehaviourEvent::Kad(event) => match event
						{
							libp2p::kad::Event::OutboundQueryProgressed {
								id,
								result,
								step,
								stats,
							} => {
								if step.last {
									match result {
										libp2p::kad::QueryResult::GetClosestPeers(result) => {
											let mut sender = self.request_handles.kad.get_mut(&id).unwrap();
											for peer in result.unwrap().peers {
												let _ =sender.send(peer);
											}
										},
										_ => {}
									}
								}
							}
							_ => {}
						},
						*/
						_ => {}
					}
				}
				_ => {}
			}
		}
		// tracing::info!("event loop yielded");
		drop(swarm_lock);

		#[cfg(feature = "client")]
		{
			let _ = gloo::time::sleep(std::time::Duration::from_secs(0)).await;
		}

		#[cfg(feature = "server")]
		{
			let _ = tokio::task::yield_now().await;
		}
	}
	}

	async fn send_request(
		&self,
		peer_id: libp2p::PeerId,
		request: SubfieldRequest,
	) -> Result<u64, SubfieldServiceError> {
		let mut swarm_lock = self.swarm.lock().await;

		let behaviour = &mut *swarm_lock.behaviour_mut();

		let request_id =
			behaviour.subfield.send_request(&peer_id, request.clone());
		let handle = get_outbound_request_id(request_id);
		let _ = self
			.request_handles
			.subfield
			.create_oneshot_with_handle(handle);

		/*
		let mut discriminated_receiver: Option<SubfieldResponse> =
			None;

		let discriminated_sender = match request.proto().request_type.clone() {
			Some(proto::subfield_request::RequestType::Ping(request)) => {
				let (sender, receiver) = portal();
				self.request_handles.ping.insert(request_id, sender.clone());
				SubfieldResponseRPC::Ping(receiver)
			}
			Some(proto::subfield_request::RequestType::Echo(request)) => {
				let (sender, receiver) = portal();
				self.request_handles.echo.insert(request_id, sender.clone());
				SubfieldResponseRPC::Echo(receiver)
			}
			_ => {
				panic!("Unsupported request type");
			},
		};
		*/

		Ok(handle)
	}

	/*
		pub async fn random_connected_peer(&self) -> Option<libp2p::PeerId> {
			let swarm_lock = self.swarm.lock().await;
			let swarm = &*swarm_lock;
			let peers: Vec<_> = swarm.connected_peers().collect();
			peers
				.into_iter()
				.choose(&mut thread_rng())
				.map(|peer| peer.clone())
		}
	*/

	pub async fn closest_local_peer(&self, target: V256) -> libp2p::PeerId {
		let mut swarm_lock = self.swarm.lock().await;
		let kad = &mut swarm_lock.behaviour_mut().kad;

		// kad.get_closest_local_peers(target.data().to_vec())
		let key = target.to_key();
		let peers = kad.get_closest_local_peers(&key);
		let peer_id = peers
			.into_iter()
			.map(|key| key.preimage().clone())
			.next()
			.unwrap();
		peer_id
	}

	/*
	pub async fn closest_global_peer(
		&self,
		target: libp2p::kad::RecordKey,
	) -> Option<libp2p::PeerId> {

		// get the swarm lock
		let mut swarm_lock = self.swarm.lock().await;
		let kad = &mut swarm_lock.behaviour_mut().kad;


		// send the get closest peers query
		let query_id = kad.get_closest_peers(target.to_vec());
		let (mut sender, mut receiver) = portal();
		self.request_handles.kad.insert(query_id, sender);

		// await the response
		let peer_id = receiver.next().await.unwrap();

		// dial the peer
		let _ = swarm_lock.dial(peer_id);

		// return the now-connected peer
		Some(peer_id)
	}
	*/
}

#[async_trait]
impl protocol::SubfieldService for SubfieldClient {
	/*
	async fn ping(
		&self,
		request: proto::PingRequest,
	) -> protocol::SubfieldServiceResult<proto::PingResponse> {
		self.send_request(proto::SubfieldRequest {
			request_type: Some(
				proto::subfield_request::RequestType::PingRequest(request),
			),
		})
	}
	*/

	async fn echo(
		self,
		request: proto::EchoRequest,
	) -> protocol::SubfieldServiceResult<proto::EchoResponse> {
		let peer_id = self
			.closest_local_peer(self.config.keypair.public_key().v256().clone())
			.await;

		let request: SubfieldRequest = SubfieldRequest::new(
			proto::subfield_request::RequestType::Echo(request),
		);

		let handle = self.send_request(peer_id, request).await?;

		let res = self
			.request_handles
			.subfield
			.recv_oneshot(handle)
			.await
			.unwrap();
		let ResponseType::Echo(res) = res.proto_type() else {
			return Err(SubfieldServiceError::UnexpectedResponseType);
		};

		Ok(res.clone())
		// Ok(proto::EchoResponse {
		// 	message: "echo".to_string(),
		// })
	}

	/*
		fn get_record(
				&self,
				request: proto::GetRecordRequest,
			) -> Result<Receiver<proto::GetRecordResponse>, SubfieldServiceError> {
			todo!()
		}

		fn delete_record(
				&self,
				request: proto::DeleteRecordRequest,
			) -> Result<proto::DeleteRecordResponse, SubfieldServiceError> {
			todo!()
		}

		fn put_record(
				&self,
				request: proto::PutRecordRequest,
			) -> Result<proto::PutRecordResponse, SubfieldServiceError> {
			todo!()
		}

		fn subscribe(
				&self,
				request: proto::SubscribeRequest,
			) -> Result<proto::SubscribeResponse, SubfieldServiceError> {
			todo!()
		}

	t	fn unsubscribe(
				&self,
				request: proto::UnsubscribeRequest,
			) -> Result<proto::UnsubscribeResponse, SubfieldServiceError> {
			todo!()
		}
		*/
}
