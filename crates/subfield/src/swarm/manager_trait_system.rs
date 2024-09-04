use std::num::NonZero;

use crate::*;

#[async_trait]
impl SubfieldSystemTrait for SwarmManager {

	// peer is guaranteed to be connected
	async fn closest_global_peer(&self, key: &VersionedBytes) -> Result<PeerId, SubfieldError> {
		let mut swarm_lock = self.swarm_lock().await;
		let query_id = swarm_lock.behaviour_mut().kad.get_n_closest_peers(key.to_vec(), NonZero::new(1).unwrap());
		drop(swarm_lock);
		
		let (tx, mut rx) = unbounded();
		self.kad_queries().insert(query_id, tx);
		
		let peer_id = rx.next().await.unwrap();
		
		Ok(peer_id)
	}

	
	async fn event_loop(&self) {
		loop {
			let mut swarm_lock = self.swarm_lock().await;

			while let Some(Some((peer, mut stream))) =
				self.incoming_streams().await.next().now_or_never()
			{
				// match recv::<SubfieldRequest>(&mut stream).await {
				// 	Ok(req) => {
						
				// 		// send(n, &mut stream).await.unwrap();
				// 	}
				// 	Err(e) => {
				// 		tracing::warn!(%peer, "Echo failed: {e}");
				// 		continue;
				// 	}
				// };
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
						swarm::SubfieldNetworkBehaviourEvent::Kad(event) => {
							match event {
								libp2p::kad::Event::OutboundQueryProgressed { id, result, stats, step } => {
									if step.last {
										match result {
											libp2p::kad::QueryResult::GetClosestPeers(closest_peers) => {
												match closest_peers {
													Ok(closest_peers) => {
														
														// get the absolute closest peer
														let closest_peer = closest_peers.peers[0].clone();
														
														// dial the closest peer if we are not already connected
														if !swarm_lock.is_connected(&closest_peer.peer_id) {
															for multiaddr in closest_peer.addrs.clone() {
																let _ = swarm_lock.dial(multiaddr);
															}
														}														
																
														// send the result to the query
														let (id, mut tx) = self.kad_queries().remove(&id).expect("Query ID not found");
														let _ = tx.send(closest_peer.peer_id);
														
													}
													Err(e) => {
														tracing::error!("Kad query error: {:?}", e);
													}
												}
											}
											_ => {}
										}
									}
								}
								_ => {}
							}
						}
						
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