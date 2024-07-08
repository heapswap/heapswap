#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]
use futures::{prelude::*, stream::StreamExt};
use libp2p::identity::ed25519::Keypair;
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;
use libp2p::{
	Swarm,
    mdns, noise, gossipsub,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
	kad::{QueryResult as KadQueryResult},
};
use std::error::Error;
use std::time::Duration;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::{net::TcpListener, io, io::AsyncBufReadExt, select, sync::{Mutex, MutexGuard}};
use bytes::Bytes;
use crate::{bys, crypto::keys::KeyPair};


#[derive(NetworkBehaviour)]
struct Behaviour {
	kademlia: kad::Behaviour<MemoryStore>,
	gossipsub: gossipsub::Behaviour,
	//mdns: mdns::tokio::Behaviour,
}

struct SwarmConfig {
	keypair: KeyPair,
	listen_addresses: Vec<String>,
	is_server: bool,
}


fn create_swarm(swarm_config:SwarmConfig ) -> Result<Swarm<Behaviour>, Box<dyn Error>> {
	
	let keypair: Keypair = Keypair::generate();
	
	
	//let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.into())
	let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        //.with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {

			// To content-address message, we can take the hash of message and use it as an ID.
			let _message_id_fn = |message: &gossipsub::Message| {
				let mut s = DefaultHasher::new();
				message.data.hash(&mut s);
				gossipsub::MessageId::from(s.finish().to_string())
			};

			// Set a custom gossipsub configuration
			let gossipsub_config = gossipsub::ConfigBuilder::default()
				.heartbeat_interval(Duration::from_secs(10))
				.validation_mode(gossipsub::ValidationMode::Strict) 
				//.message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
				.build()
				.map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

			// build a gossipsub network behaviour
			let gossipsub = gossipsub::Behaviour::new(
				gossipsub::MessageAuthenticity::Signed(key.clone()),
				gossipsub_config,
			)?;
			
            Ok(Behaviour {
				gossipsub,
                kademlia: kad::Behaviour::new(
                    key.public().to_peer_id(),
                    MemoryStore::new(key.public().to_peer_id()),
                ),
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?,
            })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
		
	
	// set mode
	if swarm_config.is_server {
		swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
	} else {
		swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Client));
	}

	// listen on addresses
	for addr in swarm_config.listen_addresses {
		swarm.listen_on(addr.parse()?)?;
	}
	
	Ok(swarm)
}

fn swarm_handle_event(swarm: &mut Swarm<Behaviour>, event: SwarmEvent<BehaviourEvent>){
	match event {
		SwarmEvent::NewListenAddr { address, .. } => {
			println!("Listening on {address:?}");
		},
		SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
			for (peer_id, multiaddr) in list {
				println!("mDNS discovered a new peer: {peer_id}");
				swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
				swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
			}
		}, 
		SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
			for (peer_id, multiaddr) in list {
				println!("mDNS discover peer has expired: {peer_id}");
				swarm.behaviour_mut().kademlia.remove_address(&peer_id, &multiaddr);
				swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
			}
		},
		SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub_event)) => {
			match gossipsub_event {
				gossipsub::Event::Message {
					propagation_source: peer_id,
					message_id: id,
					message,
				} => {
					println!(
						"Got message: '{}' with id: {id} from peer: {peer_id}",
						String::from_utf8_lossy(&message.data),
						// Additional processing can be done here
					);
				},
				gossipsub::Event::Subscribed { peer_id, topic } => {
					println!("Peer {peer_id} subscribed to topic {topic}");
				},
				gossipsub::Event::Unsubscribed { peer_id, topic } => {
					println!("Peer {peer_id} unsubscribed from topic {topic}");
				},
				// Handle other gossipsub events as needed
				_ => {}
			}
		},
		SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { result, ..})) => {
			match result {
				kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { key, providers, .. })) => {
					for peer in providers {
						println!(
							"Peer {peer:?} provides key {:?}",
							std::str::from_utf8(key.as_ref()).unwrap()
						);
					}
				}
				kad::QueryResult::GetProviders(Err(err)) => {
					eprintln!("Failed to get providers: {err:?}");
				}
				kad::QueryResult::GetRecord(Ok(
					kad::GetRecordOk::FoundRecord(kad::PeerRecord {
						record: kad::Record { key, value, .. },
						..
					})
				)) => {
					println!(
						"Got record {:?} {:?}",
						std::str::from_utf8(key.as_ref()).unwrap(),
						std::str::from_utf8(&value).unwrap(),
					);
				}
				kad::QueryResult::GetRecord(Ok(_)) => {}
				kad::QueryResult::GetRecord(Err(err)) => {
					eprintln!("Failed to get record: {err:?}");
				}
				kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
					println!(
						"Successfully put record {:?}",
						std::str::from_utf8(key.as_ref()).unwrap()
					);
				}
				kad::QueryResult::PutRecord(Err(err)) => {
					eprintln!("Failed to put record: {err:?}");
				}
				kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
					println!(
						"Successfully put provider record {:?}",
						std::str::from_utf8(key.as_ref()).unwrap()
					);
				}
				kad::QueryResult::StartProviding(Err(err)) => {
					eprintln!("Failed to put provider record: {err:?}");
				},
				
				_ => {}
			}
		}
		_ => {}
	}
}