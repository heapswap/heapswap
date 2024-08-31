use libp2p::{request_response::OutboundRequestId, Stream};
use tracing::event;
use std::{io, time::Duration};
use crate::*;

// pub enum LocalClosestKeyResult {
// 	SelfIsClosest,
// 	Ok(libp2p::PeerId),
// 	Err(SubfieldError),
// }

#[async_trait]
impl SubfieldEventsTrait for SubfieldClient {
	async fn bootstrap(&self) -> Result<(), SubfieldError> {
		let mut config = self.config().clone();

		// if the list of multiaddrs is empty, try to get it from the urls
		if config.bootstrap_multiaddrs.is_empty() {
			// if the list of urls is empty, return an error
			if config.bootstrap_urls.is_empty() {
				return Err(SubfieldError::BootstrapFailedNoUrls);
			} else {
				config = config
					.get_bootstrap_multiaddrs_from_urls()
					.await
					.map_err(|e| SubfieldError::BootstrapFailedNoMultiaddrs)?;
			}
		}

		// we only need to try to connect to one, so break after the first success
		let mut success = false;

		let mut swarm_lock = self.swarm_lock().await;

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
			Err(SubfieldError::BootstrapFailedDial)
		}
	}

	async fn event_loop(&self) {
		loop {
			let mut swarm_lock = self.swarm_lock().await;
			
			while let Some(Some((peer, stream))) = self.incoming_streams().await.next().now_or_never() {
				match echo(stream).await {
					Ok(n) => {
						tracing::info!(%peer, "Echoed {n} bytes!");
					}
					Err(e) => {
						tracing::warn!(%peer, "Echo failed: {e}");
						continue;
					}
				};
			}

			let Some(Some(event)) = swarm_lock.next().now_or_never() else {
				continue;
			};

			#[allow(unused_mut)]
			let mut behaviour: &mut swarm::SubfieldBehaviour =
				swarm_lock.behaviour_mut();

			match event {
				// swarm::SubfieldSwarmEvent::Behaviour(event) => {
				// 	match event {
				// 		SubfieldBehaviourEvent::Subfield(event) => {
							
				// 		}
						
				// 		_ => {}
				// 	}
				// }
		
				
				swarm::SubfieldSwarmEvent::Behaviour(event) => {
					match event {
					/*
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
											
											match self.closest_local_peer(&request.routing_key).await {
												Ok(Some(peer_id)) => {
													let res = self.send_request_to_closest_local_peer(request).await.unwrap();
												}
												Ok(None) => {
													match request {
														SubfieldRequest::Echo(request) => {
															let _ = self.handle_request(request_id, request, channel, swarm_lock).await;
														}
														_ => {}
													}
												}
												Err(e) => {
													tracing::error!("Error finding local peer: {:?}", e);
												}
											}
										}
										// an incoming response
										libp2p::request_response::Message::Response {
											request_id,
											response,
										} => {
											
											// let handle = Self::strip_outbound_request_id(request_id);
											self.recv_response_from_swarm(request_id, response);
										}
									}
								} 
								_ => {}
							}
						}
					*/
						#[cfg(feature = "server")]
						swarm::SubfieldBehaviourEvent::Mdns(event) => match event {
							libp2p::mdns::Event::Discovered(peer_id) => {
								for (peer_id, multiaddr) in peer_id {
									let _ = swarm_lock.dial(multiaddr);
								}
							}
							libp2p::mdns::Event::Expired(peer_id) => {
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

	/*
	Peer selection
	*/



	// most common case: closest local peer
	async fn closest_local_peer(
		&self,
		key: &RoutingKey,
	) -> Result<Option<libp2p::PeerId>, SubfieldError> {
		let routing_field = key.get_routing_field()?;
		let mut swarm_lock = self.swarm_lock().await;
		let kad = &mut swarm_lock.behaviour_mut().kad;
		let routing_field_key = routing_field.to_key();

		// get closest key
		let closest_key =
			kad.get_closest_local_peers(&routing_field_key).next();
		if closest_key.is_none() {
			return Ok(None);
		}
		let closest_key = closest_key.unwrap();
		let closest_peer_id = closest_key.preimage();

		// server needs to check if it is the closest peer
		#[cfg(feature = "server")]
		{
			let local_dist = routing_field_key.distance(self.local_peer_key());
			let closest_dist = routing_field_key.distance(&closest_key);

			if closest_dist > local_dist {
				return Ok(Some(closest_peer_id.clone()));
			}
		}

		Ok(Some(closest_peer_id.clone()))
	}

	/*
	Send requests
	*/

	/*
	async fn send_request_to_local_peer(
		&self,
		peer_id: libp2p::PeerId,
		request: SubfieldRequest,
	) -> Result<OutboundRequestId, SubfieldError> {
		let mut swarm_lock = self.swarm_lock().await;

		let behaviour = &mut *swarm_lock.behaviour_mut();

		// let request_id =
		// 	behaviour.subfield.send_request(&peer_id, request.clone());

		// Ok(request_id)
	}

	async fn send_request_to_closest_local_peer(
		&self,
		request: SubfieldRequest,
	) -> Result<OutboundRequestId, SubfieldError> {
		let peer_id = match self.closest_local_peer(&request.routing_key).await {
			Ok(Some(peer_id)) => peer_id,
			Ok(None) => {
				return Err(SubfieldError::SelfIsClosest);
			}
			Err(e) => {
				return Err(e);
			}
		};

		self.send_request_to_local_peer(peer_id, request).await
	}
	*/

}


/// A very simple, `async fn`-based connection handler for our custom echo protocol.
async fn connection_handler(peer: PeerId, mut control: stream::Control) {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await; // Wait a second between echos.

        let stream = match control.open_stream(peer, SUBFIELD_PROTOCOL).await {
            Ok(stream) => stream,
            Err(error @ stream::OpenStreamError::UnsupportedProtocol(_)) => {
                tracing::info!(%peer, %error);
                return;
            }
            Err(error) => {
                // Other errors may be temporary.
                // In production, something like an exponential backoff / circuit-breaker may be more appropriate.
                tracing::debug!(%peer, %error);
                continue;
            }
        };

        if let Err(e) = send(stream).await {
            tracing::warn!(%peer, "Echo protocol failed: {e}");
            continue;
        }

        tracing::info!(%peer, "Echo complete!")
    }
}

async fn echo(mut stream: Stream) -> io::Result<usize> {
    let mut total = 0;

    let mut buf = [0u8; 100];

    loop {
        let read = stream.read(&mut buf).await?;
        if read == 0 {
            return Ok(total);
        }

        total += read;
        stream.write_all(&buf[..read]).await?;
    }
}

async fn send(mut stream: Stream) -> io::Result<()> {
    let num_bytes = rand::random::<usize>() % 1000;

    let mut bytes = vec![0; num_bytes];
    rand::thread_rng().fill_bytes(&mut bytes);

    stream.write_all(&bytes).await?;

    let mut buf = vec![0; num_bytes];
    stream.read_exact(&mut buf).await?;

    if bytes != buf {
        return Err(io::Error::new(io::ErrorKind::Other, "incorrect echo"));
    }

    stream.close().await?;

    Ok(())
}