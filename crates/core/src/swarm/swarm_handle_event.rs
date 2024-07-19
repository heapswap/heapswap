use crate::subfield::*;
use crate::swarm::*;
#[cfg(not(target_arch = "wasm32"))]
use libp2p::mdns;
use libp2p::{gossipsub, kad, ping, swarm::SwarmEvent, Swarm};

#[derive(Debug)]
pub enum SwarmHandleEventError {
	FailedToLockSwarm,
}

pub async fn swarm_handle_event(
	swarm: &mut SubfieldSwarm,
	event: SubfieldSwarmEvent,
) -> Result<(), SwarmHandleEventError> {

	match event {
		SwarmEvent::Behaviour(swarm_event) => {
			match swarm_event {
				/*
				 * Ping
					*/
				SubfieldBehaviourEvent::Ping(ping_event) => match ping_event {
					ping::Event {
						peer,
						connection,
						result,
					} => match result {
						Ok(duration) => {
							tracing::info!(
								"Ping successful with {:?} - {:?}ms",
								peer,
								duration.as_millis()
							);
						}
						Err(e) => {
							tracing::info!(
								"Ping failed with {:?}: {:?}",
								peer,
								e
							);
						}
					},
				},
				/*
				 * Kademlia
					*/
				SubfieldBehaviourEvent::Kademlia(kad_event) => {
					match kad_event {
						kad::Event::OutboundQueryProgressed {
							id,
							result,
							stats,
							step,
						} => {}
						_ => {}
					}
				}
				/*
				 * Gossipsub
					*/
				SubfieldBehaviourEvent::Gossipsub(gossipsub_event) => {
					match gossipsub_event {
						_ => {}
					}
				}
				/*
				 * MDNS
					*/
				#[cfg(not(target_arch = "wasm32"))]
				SubfieldBehaviourEvent::Mdns(mdns_event) => {
					#[cfg(not(target_arch = "wasm32"))]
					match mdns_event {
						mdns::Event::Discovered(list) => {
							for (peer_id, multiaddr) in list {
								tracing::info!(
									"mDNS discovered a new peer: {:?}",
									peer_id
								);
								let _ = swarm.dial(multiaddr.clone());
								swarm
									.behaviour_mut()
									.kademlia
									.add_address(&peer_id, multiaddr);
							}
						}
						mdns::Event::Expired(list) => {
							for (peer_id, _) in list {
								tracing::info!(
									"mDNS discover peer has expired: {:?}",
									peer_id
								);
							}
						}
					}
					#[cfg(target_arch = "wasm32")]
					match mdns_event {
						_ => {}
					}
				}
			}
		}
		SwarmEvent::NewListenAddr {
			listener_id,
			address,
		} => {
			tracing::info!(
				"{:?} listening on {:?}",
				swarm.local_peer_id(),
				address
			);
		}
		SwarmEvent::ConnectionEstablished {
			peer_id,
			num_established,
			endpoint,
			..
		} => {
			tracing::info!(
				"Connection established with {:?} ({:?})",
				peer_id,
				num_established
			);
			swarm
				.behaviour_mut()
				.kademlia
				.add_address(&peer_id, endpoint.get_remote_address().clone());
		}
		SwarmEvent::ConnectionClosed {
			peer_id,
			num_established,
			..
		} => {
			tracing::info!(
				"Connection closed with {:?} ({:?})",
				peer_id,
				num_established
			);
		}
		_ => {}
	}
	Ok(())
}
