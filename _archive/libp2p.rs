#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]
use futures::{prelude::*, stream::StreamExt};
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
use tracing_subscriber::EnvFilter;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::{net::TcpListener, io, io::AsyncBufReadExt, select, sync::{Mutex, MutexGuard}};
use bytes::Bytes;
use heapswap_core::{bys};


#[derive(NetworkBehaviour)]
struct Behaviour {
	kademlia: kad::Behaviour<MemoryStore>,
	gossipsub: gossipsub::Behaviour,
	mdns: mdns::tokio::Behaviour,
}

struct SwarmConfig {
	listen_addresses: Vec<String>,
	is_server: bool,
}


fn create_swarm(swarm_config:SwarmConfig ) -> Result<Swarm<Behaviour>, Box<dyn Error>> {
	let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
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

fn handle_event(swarm: &mut Swarm<Behaviour>, event: SwarmEvent<BehaviourEvent>){
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

	let swarm = Arc::new(Mutex::new(create_swarm(SwarmConfig {
		listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".to_string()],
		is_server: true,
	})?));
	let swarm_clone = Arc::clone(&swarm);

	// Create a Gossipsub topic
	let topic = gossipsub::IdentTopic::new("test-net");
	// subscribes to our topic
	swarm.lock().await.behaviour_mut().gossipsub.subscribe(&topic)?;
	
	// Move the cloned Arc into the async block
	tokio::spawn(async move {
		let mut interval = tokio::time::interval(Duration::from_secs(5));
		loop {
			interval.tick().await;
			// Lock the mutex to access the swarm inside the async block
			let mut swarm = swarm_clone.lock().await;
			let message = heapswap_core::bys::random(32);
			let _ = swarm.behaviour_mut().gossipsub.publish(topic.clone(), message);
			println!("swarm published message");
		}
	});

	loop {
		let mut lock = swarm.lock().await;
		select! {
			event = lock.select_next_some() => {
				handle_event(&mut *lock, event)
			},
		}
	}
}

fn handle_input_line(kademlia: &mut kad::Behaviour<MemoryStore>, line: String) {
    let mut args = line.split(' ');

    match args.next() {
        Some("GET") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            kademlia.get_record(key);
        }
        Some("GET_PROVIDERS") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            kademlia.get_providers(key);
        }
        Some("PUT") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            let value = {
                match args.next() {
                    Some(value) => value.as_bytes().to_vec(),
                    None => {
                        eprintln!("Expected value");
                        return;
                    }
                }
            };
            let record = kad::Record {
                key,
                value,
                publisher: None,
                expires: None,
            };
            kademlia
                .put_record(record, kad::Quorum::One)
                .expect("Failed to store record locally.");
        }
        Some("PUT_PROVIDER") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };

            kademlia
                .start_providing(key)
                .expect("Failed to start providing key");
        }
        _ => {
            eprintln!("expected GET, GET_PROVIDERS, PUT or PUT_PROVIDER");
        }
    }
}

#[tokio::test]
async fn test_swarm() -> Result<(), Box<dyn Error>> {
	
	let replicas = 3;
	let mut swarms = Vec::new();
	
	println!("Creating {} swarms", replicas);
	
	for i in 0..replicas {
		let swarm = Arc::new(Mutex::new(create_swarm(SwarmConfig {
			listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".to_string()],
			is_server: true,
		})?));
		let swarm_clone = Arc::clone(&swarm);

		// Create a Gossipsub topic
		let topic = gossipsub::IdentTopic::new("test-net");
		// subscribes to our topic
		swarm.lock().await.behaviour_mut().gossipsub.subscribe(&topic)?;
		
		swarms.push(swarm);
		
		if i == 0 {
			// Move the cloned Arc into the async block
			tokio::spawn(async move {
				let mut interval = tokio::time::interval(Duration::from_secs(5));
				loop {
					interval.tick().await;
					// Lock the mutex to access the swarm inside the async block
					let mut swarm = swarm_clone.lock().await;
					let message = bys::to_base32(&bys::random(16));
					let _ = swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.clone());
					println!("swarm {} published message {}", swarm.local_peer_id(), message);
				}
			});
		}
	}
	
	// wait for 30s
	//tokio::time::sleep(Duration::from_secs(30)).await;
	loop { 
		for swarm in swarms.iter() {
			select! {
				event = swarm.lock().await.select_next_some() => {
					handle_event(&mut *swarm.lock().await, event)
				},
			}
		}
	}
	//Ok(())
}