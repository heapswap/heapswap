use libp2p::request_response::{InboundRequestId, OutboundRequestId};

pub use crate::*;
use futures::{TryFuture, TryFutureExt};

#[derive(Clone, Debug)]
pub struct SubfieldConfig {
	pub keypair: crypto::Keypair,

	pub mode: SubfieldSwarmMode,

	pub bootstrap_urls: Vec<String>,
	// pub bootstrap_multiaddrs: Vec<String>,
	pub bootstrap_multiaddrs: Vec<libp2p::Multiaddr>,
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
				"http://localhost:3000/bootstrap".to_string(),
				"https://heapswap.com/bootstrap".to_string(),
			],
			bootstrap_multiaddrs: vec![],
			#[cfg(feature = "server")]
			listen_addresses: vec![
				// "/ip4/0.0.0.0/tcp/3000/webrtc".to_string(),
				// "/ip6/::/tcp/3000/webrtc".to_string(),
			],
			store_path: String::from("_subfield_store"),
		}
	}
}

impl SubfieldConfig {
	pub async fn get_bootstrap_multiaddrs_from_urls(
		&mut self,
	) -> EResult<Self> {
		// let mut config = self.clone();
		// let mut bootstrap_multiaddrs = vec![];

		for url in self.bootstrap_urls.clone() {
			tracing::info!("Dialing bootstrap URL: {:?}", url);

			// get the json multiaddr list from the url
			let multiaddr_list_res = 				reqwest::get(url).await?.json::<Vec<String>>().await;
			
			match multiaddr_list_res {
				Ok(multiaddr_list) => {
					for multiaddr in multiaddr_list {
						self.bootstrap_multiaddrs
							.push(multiaddr.parse::<libp2p::Multiaddr>()?);
					}
				}
				Err(e) => {
					// failed to get the multiaddr list, try the next url
				}
			}


		}
		
		if self.bootstrap_multiaddrs.is_empty() {
			tracing::error!("Failed to get bootstrap multiaddrs from urls");
			// return Err(eyr!("Failed to get bootstrap multiaddrs from urls"));
		}

		Ok(self.clone())
	}
}

trait SizedMessage: Message + Sized {}

#[derive(Clone)]
pub struct SubfieldClient {
	local_peer_id: libp2p::PeerId,
	local_peer_key: libp2p::kad::KBucketKey<Vec<u8>>,

	config: SubfieldConfig,

	swarm: swarm::ThreadsafeSubfieldSwarm,
	// store: store::SubfieldStore,
	request_handles: Arc<SubfieldRequestHandles>,
}

type HandleMap<T> = DashMap<OutboundRequestId, Sender<T>>;

enum SubfieldRequestRPC {
	Ping(PingRequest),
	Echo(EchoRequest),
	GetRecord(GetRecordRequest),
	DeleteRecord(DeleteRecordRequest),
	PutRecord(PutRecordRequest),
	Subscribe(SubscribeRequest),
	Unsubscribe(UnsubscribeRequest),
}

enum SubfieldResponseRPC {
	Ping(PingResponse),
	Echo(EchoResponse),
	GetRecord(GetRecordResponse),
	DeleteRecord(DeleteRecordResponse),
	PutRecord(PutRecordResponse),
	Subscribe(Receiver<SubscribeResponse>),
	Unsubscribe(UnsubscribeResponse),
}

/*
struct SubfieldRequestHandles {
	// kad
	kad: DashMap<libp2p::kad::QueryId, Sender<libp2p::PeerId>>,
	// system
	ping: HandleMap<PingResponse>,
	echo: HandleMap<EchoResponse>,
	// records
	get_record: HandleMap<GetRecordResponse>,
	delete_record: HandleMap<DeleteRecordResponse>,
	put_record: HandleMap<PutRecordResponse>,
	// pubsub
	subscribe: HandleMap<SubscribeResponse>,
	unsubscribe: HandleMap<UnsubscribeResponse>,
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

pub enum LocalClosestKeyResult {
	Ok(libp2p::PeerId),
	NoPeersConnected,
	SelfIsClosest,
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
		// let store = store::SubfieldStore::new(config.clone()).await?;
		// let peer_id = libp2p::PeerId::from_public_key(libp2p::identity::PublicKey::config.keypair.public_key());
		let local_peer_id =
			config.keypair.public_key().to_libp2p_peer_id().unwrap();
		let local_peer_key = config.keypair.public_key().v256().to_key();
		Ok(Self {
			local_peer_id,
			local_peer_key,
			config,
			swarm,
			request_handles,
			// store,
		})
	}

	/**
	 * Getters
		*/
	pub async fn swarm(&self) -> MutexGuard<swarm::SubfieldSwarm> {
		self.swarm.lock().await
	}

	// pub async fn cache(
	// 	&self,
	// ) -> MutexGuard<Pin<Box<dyn store::FullStore + Send>>> {
	// 	self.store.cache().await
	// }

	// pub async fn perma(
	// 	&self,
	// ) -> MutexGuard<Pin<Box<dyn store::FullStore + Send>>> {
	// 	self.store.perma().await
	// }
	
	/**
	 * Bootstrap
	*/

	pub async fn bootstrap(&self) -> eyre::Result<()> {
		
		let mut config = self.config.clone();		
		
		// if the list of multiaddrs is empty, try to get it from the urls
		if config.bootstrap_multiaddrs.is_empty() {
			// if the list of urls is empty, return an error
			if config.bootstrap_urls.is_empty() {
				return Err(eyr!("No bootstrap URLs provided"));
			} else {
				config = config.get_bootstrap_multiaddrs_from_urls().await?;
			}
		}
		
		// we only need to try to connect to one, so break after the first success
		let mut success = false;

		let mut swarm_lock = self.swarm.lock().await;

		for multiaddr in config.bootstrap_multiaddrs.clone() {
			tracing::info!("Dialing bootstrap Multiaddr: {:?}", multiaddr);
			match swarm_lock.dial(multiaddr.clone()) {
				Ok(_) => {
					tracing::info!(
						"Successfully dialed bootstrap Multiaddr: {:?}",
						multiaddr
					);
					success = true;
					break;
				}
				Err(e) => {
					tracing::error!(
						"Failed to dial bootstrap Multiaddr: {:?}, error: {:?}",
						multiaddr,
						e
					);
				}
			}
		}

		if success {
			return Ok(());
		} else {
			Err(eyr!("Failed to connect to any bootstrap nodes"))
		}
	}

	pub async fn event_loop(&self) {
		loop {
			let mut swarm_lock = self.swarm.lock().await;

			let Some(Some(event)) = swarm_lock.next().now_or_never() else {
				continue;
			};

			#[allow(unused_mut)]
			let mut behaviour: &mut swarm::SubfieldBehaviour =
				swarm_lock.behaviour_mut();

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
										// an incoming request
										libp2p::request_response::Message::Request {
											request_id,
											request,
											channel,
										} => {
											match request {
												SubfieldRequest::Echo(request) => {
													let response = SubfieldResponse::Echo(Ok(EchoSuccess {
														message: request.message,
													}));
													behaviour.subfield.send_response(channel, response);
												}
												
												_ => {}
											}
										}
										// an incoming response
										libp2p::request_response::Message::Response {
											request_id,
											response,
										} => {
											let handle = get_outbound_request_id(request_id);
											
											self.request_handles.subfield.send_stream(handle, response);
										}
										// _ => {}
									}
								}
								_ => {}
							}
						}
						#[cfg(feature = "server")]
						swarm::SubfieldBehaviourEvent::Mdns(event) => match event {
							libp2p::mdns::Event::Discovered(peer_id) => {
								for (peer_id, multiaddr) in peer_id {
									// tracing::info!("Discovered peer: {:?}", peer_id);
									let _ = swarm_lock.dial(multiaddr);
								}
							}
							libp2p::mdns::Event::Expired(peer_id) => {
								// tracing::info!("Expired peer: {:?}", peer_id);
							}
						},
						_ => {}
					}
				}

				libp2p::swarm::SwarmEvent::ConnectionEstablished {
					peer_id,
					connection_id,
					endpoint,
					num_established,
					concurrent_dial_errors,
					established_in,
				} => match endpoint {
					libp2p_core::ConnectedPoint::Dialer {
						address,
						role_override,
						port_use,
					} => {
						tracing::info!(
							"Connection established with peer: {:?}",
							peer_id
						);
						behaviour.kad.add_address(&peer_id, address);
					}
					libp2p_core::ConnectedPoint::Listener {
						local_addr,
						send_back_addr,
					} => {
						tracing::info!(
							"Connection established with peer: {:?}",
							peer_id
						);
						behaviour.kad.add_address(&peer_id, send_back_addr);
					}
				},
				_ => {}
			}

			// yield the event loop
			drop(swarm_lock);
			#[cfg(feature = "client")]
			{
				let _ = gloo::timers::future::sleep(
					std::time::Duration::from_secs(0),
				)
				.await;
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

		Ok(handle)
	}

	async fn send_request_to_closest_local_peer(
		&self,
		request: SubfieldRequest,
	) -> Result<u64, SubfieldServiceError> {
		let peer_id = match self
			.closest_local_peer(self.config.keypair.public_key().v256().clone())
			.await
		{
			LocalClosestKeyResult::Ok(peer_id) => peer_id,
			LocalClosestKeyResult::SelfIsClosest => {
				return Err(SubfieldServiceError::SelfIsClosest)
			}
			LocalClosestKeyResult::NoPeersConnected => {
				return Err(SubfieldServiceError::NoConnectedPeers)
			}
		};

		self.send_request(peer_id, request).await
	}

	/**
	 * Peer selection
		*/

	// will only be used for testing
	pub async fn random_local_peer(&self) -> Option<libp2p::PeerId> {
		let swarm_lock = self.swarm.lock().await;
		let swarm = &*swarm_lock;
		let peers: Vec<_> = swarm.connected_peers().collect();
		peers
			.into_iter()
			.choose(&mut thread_rng())
			.map(|peer| peer.clone())
	}

	// most common case: closest local peer
	pub async fn closest_local_peer(
		&self,
		target: V256,
	) -> LocalClosestKeyResult {
		let mut swarm_lock = self.swarm.lock().await;
		let kad = &mut swarm_lock.behaviour_mut().kad;
		// let target_key = target.v256().to_key();
		let target_key = target.to_key();

		// get closest key
		let closest_key = kad.get_closest_local_peers(&target_key).next();
		if closest_key.is_none() {
			return LocalClosestKeyResult::NoPeersConnected;
		}
		let closest_key = closest_key.unwrap();
		let closest_peer_id = closest_key.preimage();

		let local_dist = target_key.distance(&self.local_peer_key);
		let closest_dist = target_key.distance(&closest_key);

		if closest_dist < local_dist {
			return LocalClosestKeyResult::Ok(closest_peer_id.clone());
		} else {
			return LocalClosestKeyResult::SelfIsClosest;
		}
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
impl SubfieldService for SubfieldClient {
	/*
	async fn ping(
		&self,
		request: PingRequest,
	) -> SubfieldServiceResult<PingResponse> {
		self.send_request(SubfieldRequest {
			request_type: Some(
				subfield_request::SubfieldRequest::PingRequest(request),
			),
		})
	}
	*/

	async fn echo(
		&self,
		message: &str,
	) -> SubfieldServiceResult<EchoResponse> {
		
		// let request = EchoRequest {
		// 	message: message.to_string(),
		// };	
		
		// let handle = self
		// 	.send_request_to_closest_local_peer(SubfieldRequest::Echo(request))
		// 	.await?;

		// // await the response
		// match self.request_handles.subfield.recv_oneshot(handle).await {
		// 	Ok(res) => {
		// 		tracing::info!("Received echo response: {:?}", res);

		// 		let SubfieldResponse::Echo(res) = res.proto_type() else {
		// 			return Err(SubfieldServiceError::UnexpectedSubfieldResponse);
		// 		};

		// 		Ok(res.clone())
		// 	}
		// 	Err(e) => {
		// 		tracing::error!("Failed to receive echo response: {:?}", e);
		// 		Err(SubfieldServiceError::UnexpectedSubfieldResponse)
		// 	}
		// }
		
		todo!()
	}

	async fn get_record(
		&self,
		subkey: Subkey,
	) -> Result<GetRecordResponse, SubfieldServiceError> {
		
		// let requests = subkey.to_get_record_requests().map_err(|_| SubfieldServiceError::IncompleteSubkey)?;
		
		// // send each of the requests to their closest local peers
		// // each handle returns a oneshot channel (impl Future)
		// let channels = requests.map(|request| {
		// 	let future = async move {
		// 		let handle = self.send_request_to_closest_local_peer(SubfieldRequest::GetRecord(request)).await?;
		// 		let channel = self.request_handles.subfield.recv_oneshot(handle);
		// 		channel.await.map_err(|_| SubfieldServiceError::UnexpectedSubfieldResponse)
		// 	};
		// 	Box::pin(future)
		// });
		
		// // wait for any of the channels to return a response
		// let responses = futures::future::select_ok(channels).await?;
		
		// match responses.0.proto_type() {
		// 	SubfieldResponse::GetRecord(res) => {
		// 		Ok(res.clone())
		// 	}
		// 	_ => {
		// 		Err(SubfieldServiceError::UnexpectedSubfieldResponse)
		// 	}
		// }
		
		todo!()
	}
	
	async fn put_record(
		&self,
		subkey: Subkey,
		record: Record,
	) -> Result<PutRecordResponse, SubfieldServiceError> {
			
			// let requests = subkey.to_put_record_requests(&self.config.keypair, record).map_err(|_| SubfieldServiceError::IncompleteSubkey)?;
			
			// // send each of the requests to their closest local peers
			// // each handle returns a oneshot channel (impl Future)
			// let channels = requests.map(|request| {
			// 	let future = async move {
			// 		let handle = self.send_request_to_closest_local_peer(SubfieldRequest::GetRecord(request)).await?;
			// 		let channel = self.request_handles.subfield.recv_oneshot(handle);
			// 		channel.await.map_err(|_| SubfieldServiceError::UnexpectedSubfieldResponse)
			// 	};
			// 	Box::pin(future)
			// });
			
			// // wait for any of the channels to return a response
			// let responses = futures::future::select_ok(channels).await?;
			
			// match responses.0.proto_type() {
			// 	SubfieldResponse::GetRecord(res) => {
			// 		Ok(res.clone())
			// 	}
			// 	_ => {
			// 		Err(SubfieldServiceError::UnexpectedSubfieldResponse)
			// 	}
			// }
		
		todo!()
	}
	
	async fn put_record_with_keypair(
		&self,
		keypair: Keypair,
		subkey: Subkey,
		record: Record,
	) -> Result<PutRecordResponse, SubfieldServiceError> {
		todo!()
	}
	
	
	
	/*
		fn delete_record(
				&self,
				request: DeleteRecordRequest,
			) -> Result<DeleteRecordResponse, SubfieldServiceError> {
			todo!()
		}

		fn put_record(
				&self,
				request: PutRecordRequest,
			) -> Result<PutRecordResponse, SubfieldServiceError> {
			todo!()
		}

		fn subscribe(
				&self,
				request: SubscribeRequest,
			) -> Result<SubscribeResponse, SubfieldServiceError> {
			todo!()
		}

	t	fn unsubscribe(
				&self,
				request: UnsubscribeRequest,
			) -> Result<UnsubscribeResponse, SubfieldServiceError> {
			todo!()
		}
		*/
}
