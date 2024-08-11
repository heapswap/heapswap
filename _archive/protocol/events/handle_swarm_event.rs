use super::*;
use crate::*;
#[cfg(feature = "server")]
use libp2p::mdns;
use libp2p::request_response::Event as SubfieldEvent;
use libp2p::{gossipsub, kad, ping, swarm::SwarmEvent, Swarm};

pub async fn handle_swarm_event(
	mut store: &mut store::SubfieldStore,
	mut swarm: &mut swarm::SubfieldSwarm,
	mut tx: &mut Transmitter<SubfieldRequest>,
	event: swarm::SubfieldSwarmEvent,
) -> Result<(), SwarmHandleEventError> {
	
	let key = kad::RecordKey::from(vec![0; 32]);
	let value = vec![0; 32];
	let record = kad::Record::new(key.clone(), value);
	
	let _ =	swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One);
	let _ = swarm.behaviour_mut().kademlia.get_record(key.clone());
	let _ = swarm.behaviour_mut().kademlia.remove_record(&key);
	
	let _ = swarm.behaviour_mut().kademlia.pu
	
	let query_id = swarm.behaviour_mut().kademlia.get_closest_peers(key.to_vec());
	
	let kkey = kad::KBucketKey::from(key.to_vec());
	
	let peers = swarm.behaviour_mut().kademlia.get_closest_local_peers(&kkey).next();
	
	// let self_peer = swarm.behaviour_mut().kademlia.
	
	let peer = peers.unwrap();
	let peer = peer.preimage();
	
	let request = SubfieldRequest::Get(GetRequest { topic : SubfieldTopic::new()}) ;
	let response = SubfieldResponse::Get(GetResponse::Success(vec![0; 32]));
	
	let req_id = swarm.behaviour_mut().subfield.send_request(peer, request);
	
	
	match event {
		SwarmEvent::Behaviour(swarm_event) => {
			match swarm_event {		
				/*
				 * Subfield
				 	*/
				swarm::SubfieldBehaviourEvent::Subfield(subfield_event) => {
					match subfield_event {
						SubfieldEvent::Message { peer, message } => {
							match message {
								libp2p::request_response::Message::Request { request_id, request, channel }
								=> {
									
								}
								libp2p::request_response::Message::Response { request_id, response } => {
									
								}
							}
						}
						_ => {}
					}
				}

				/*
				 * Ping
					*/
				swarm::SubfieldBehaviourEvent::Ping(ping_event) => {
					match ping_event {
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
					}
				}
				/*
				 * Kademlia
					*/
				/*
				swarm::SubfieldBehaviourEvent::Kademlia(kad_event) => {
					match kad_event {
						kad::Event::OutboundQueryProgressed {
							id,
							result,
							stats,
							step,
						} => {
							handle_kad_outbound_progressed(id, result, stats, step);
						}
						kad::Event::InboundRequest { request } => {
							handle_kad_inbound_request(request);
						}
						_ => {}
					}
				}
				*/
				/*
				 * Gossipsub
					*/
				swarm::SubfieldBehaviourEvent::Gossipsub(gossipsub_event) => {
					match gossipsub_event {
						_ => {}
					}
				}
				/*
				 * MDNS
					*/
				#[cfg(feature = "server")]
				swarm::SubfieldBehaviourEvent::Mdns(mdns_event) => {
					#[cfg(feature = "server")]
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
					#[cfg(feature = "browser")]
					match mdns_event {
						_ => {}
					}
				}
				_ => {}
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
