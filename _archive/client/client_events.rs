use crate::*;
use libp2p::{request_response::OutboundRequestId, Stream};
use std::{io, time::Duration};
use tracing::event;

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

			while let Some(Some((peer, mut stream))) =
				self.incoming_streams().await.next().now_or_never()
			{
				match recv::<SubfieldRequest>(&mut stream).await {
					Ok(req) => {
						
						// send(n, &mut stream).await.unwrap();
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
			let mut behaviour: &mut swarm::SubfieldNetworkBehaviour =
				swarm_lock.behaviour_mut();

			match event {
				swarm::SubfieldSwarmEvent::Behaviour(event) => {
					match event {
						#[cfg(feature = "server")]
						swarm::SubfieldNetworkBehaviourEvent::Mdns(event) => match event {
							libp2p::mdns::Event::Discovered(peer_id) => {
								for (peer_id, multiaddr) in peer_id {
									let _ = swarm_lock.dial(multiaddr);
								}
							}
							libp2p::mdns::Event::Expired(peer_id) => {}
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
				libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => {
					match endpoint {
						libp2p_core::ConnectedPoint::Dialer { address, role_override, port_use } => {
							behaviour.kad.remove_address(&peer_id, &address);
						}
						libp2p_core::ConnectedPoint::Listener { local_addr, send_back_addr } => {
							behaviour.kad.remove_address(&peer_id, &send_back_addr);
						}
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
	
}