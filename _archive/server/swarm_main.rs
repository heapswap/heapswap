

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