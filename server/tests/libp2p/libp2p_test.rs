use {
	libp2p::{
		kad,
		kad::{store::MemoryStore, Mode},
		mdns, noise,
		swarm::{NetworkBehaviour, SwarmEvent},
		tcp, yamux, SwarmBuilder,
	},
	std::error::Error,
	std::time::Duration,
	tracing_subscriber::EnvFilter,
	futures::{prelude::*, select},
	std::sync::Arc,
	tokio::sync::Mutex,
};


#[tokio::test]
async fn main() -> Result<(), Box<dyn Error>>{
    let mut swarms = Vec::new();

    // Create 3 swarm nodes
    for port in 5000..5003 {
        let test_swarm = test_swarm_builder(port).await?;
        swarms.push(Arc::new(Mutex::new(test_swarm)));
    }

    // Iterate over each swarm and call the event matcher function
    let mut handles = Vec::new();
    for swarm in &swarms {
        let swarm_clone = Arc::clone(swarm);
        let handle = tokio::spawn(async move {
            loop {
                // Limit the scope of the lock
                {
                    let mut locked_swarm = swarm_clone.lock().await;
                    select! {
                        event = locked_swarm.select_next_some() => event_matcher(&mut *locked_swarm, event).await,
                    }
                }
                // Sleep to allow other tasks to acquire the lock
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        handles.push(handle);
    }
	
	// wait 10 seconds
	tokio::time::sleep(Duration::from_secs(5)).await;
	
	println!("done waiting");
    
    // Get a mutable reference to the first swarm
	let locked_swarm = &mut swarms[0].lock().await;
    let kademlia = &mut locked_swarm.behaviour_mut().kademlia;
	
	println!("lock acquired");
	
	kademlia.put_record(kad::Record {
		key: kad::RecordKey::new(b"hello"),
		value: b"world".to_vec(),
		publisher: None,
		expires: None,
	}, kad::Quorum::One);
	
	println!("Putting record");
    
	
    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    Ok(())
}



async fn event_matcher(swarm: &mut libp2p::Swarm<Behaviour>,event: SwarmEvent<BehaviourEvent>) {
	match event {
		
		// listening
		SwarmEvent::NewListenAddr { address, .. } => {
			println!("Listening on {address:?}");
		},
		SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
			for (peer_id, multiaddr) in list {
				swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
			}
		}
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
				}
				_ => {}
			}
		},		
		
		// default handler
		_  => {}				
	}
}


// We create a custom network behaviour that combines Kademlia and mDNS.
#[derive(NetworkBehaviour)]
struct Behaviour {
	kademlia: kad::Behaviour<MemoryStore>,
	mdns: mdns::async_io::Behaviour,
}

async fn test_swarm_builder(
	port: u16
) -> Result<libp2p::Swarm<Behaviour>, Box<dyn std::error::Error>> {
	let mut swarm = libp2p::SwarmBuilder::with_new_identity()
		.with_async_std()
		.with_tcp(
			tcp::Config::default(),
			noise::Config::new,
			yamux::Config::default,
		)?
		.with_behaviour(|key| {
			Ok(Behaviour {
				kademlia: kad::Behaviour::new(
					key.public().to_peer_id(),
					MemoryStore::new(key.public().to_peer_id()),
				),
				mdns: mdns::async_io::Behaviour::new(
					mdns::Config::default(),
					key.public().to_peer_id(),
				)?,
			})
		})?
		.with_swarm_config(|c| {
			c.with_idle_connection_timeout(Duration::from_secs(60))
		})
		.build();

	swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));

	swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;
	
	return Ok(swarm);
}
