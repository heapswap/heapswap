use crate::*;


#[async_trait]
impl SubfieldEventsTrait for SubfieldClient { 
	
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
													let _ = behaviour.subfield.send_response(channel, response);
												}
												SubfieldRequest::PutRecord(request) => {
													// check if self is the closest peer
													let closest_peer = self.closest_local_peer(request.routing_subkey.clone()).await;
													
													match closest_peer {
														
														// self is the closest peer
														LocalClosestKeyResult::SelfIsClosest => {
															
															// verify the signature
															match request.verify() {
																Ok((subkey, record)) => {
																	// put the record locally
																	self.store.insert(subkey, record);
																	
																	// send a success response
																	let _ = behaviour.subfield.send_response(channel, SubfieldResponse::PutRecord(Ok(PutRecordSuccess {})));
																}
																Err(e) => {
																	// return an error
																	let _ = behaviour.subfield.send_response(channel, SubfieldResponse::PutRecord(Err(PutRecordFailure::RecordError(e))));
																}
															}
															
														}
														
														// a remote peer is closest
														LocalClosestKeyResult::Ok(peer_id) => {
															// handoff to remote peer
														}
													}
													
													
													
													
												}
												_ => {}
											}
										}
										// an incoming response
										libp2p::request_response::Message::Response {
											request_id,
											response,
										} => {
											let mut oneshot = false;
											
											let handle = get_outbound_request_id(request_id);
											
											let _ = self.send_response_to_portal(handle, response);
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

	pub async fn send_request(
		&self,
		peer_id: libp2p::PeerId,
		request: SubfieldRequest,
	) -> Result<u64, SubfieldError> {

		let mut swarm_lock = self.swarm.lock().await;

		let behaviour = &mut *swarm_lock.behaviour_mut();

		let request_id =
			behaviour.subfield.send_request(&peer_id, request.clone());

		let handle = get_outbound_request_id(request_id);
		
		self.create_portal(handle, &request)?;

		Ok(handle)
	}

	pub async fn send_request_to_closest_local_peer(
		&self,
		subkey: RoutingSubkey,
		request: SubfieldRequest,
	) -> Result<u64, SubfieldError> {
		
		let peer_id = match self
			.closest_local_peer(subkey)
			.await
		{
			LocalClosestKeyResult::Ok(peer_id) => peer_id,
			LocalClosestKeyResult::SelfIsClosest => {
				return Err(SubfieldError::SelfIsClosest)
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
		subkey: RoutingSubkey,
	) -> LocalClosestKeyResult {
		let target = subkey.get_only_routing_field();
		let mut swarm_lock = self.swarm.lock().await;
		let kad = &mut swarm_lock.behaviour_mut().kad;
		let target_key = target.to_key();

		// get closest key
		let closest_key = kad.get_closest_local_peers(&target_key).next();
		if closest_key.is_none() {
			return LocalClosestKeyResult::SelfIsClosest;
		}
		let closest_key = closest_key.unwrap();
		let closest_peer_id = closest_key.preimage();

		let local_dist = target_key.distance(&self.local_peer_key);
		let closest_dist = target_key.distance(&closest_key);


		// server needs to check if it is the closest peer
		#[cfg(feature = "server")]
		{
			if closest_dist > local_dist {
				return LocalClosestKeyResult::Ok(closest_peer_id.clone());
			}
		}
		
		return LocalClosestKeyResult::SelfIsClosest;
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