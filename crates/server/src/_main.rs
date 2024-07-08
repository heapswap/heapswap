#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]
#![allow(unused_braces)]
#![allow(unused_variables)]

use axum::extract::State;
use axum::Json;
use futures::StreamExt;
use heapswap_core::{bys, networking::subfield::*};
use heapswap_server::swarm::*;

use libp2p::swarm::handler::multi;
use libp2p::swarm::SwarmEvent;
use libp2p::Swarm;
use libp2p::{
	identity::ed25519::Keypair, request_response::ProtocolSupport,
	StreamProtocol,
};
use tokio::net::TcpListener;

use std::error::Error;
use std::ops::Sub;
use std::sync::Arc;
use std::net::SocketAddr;

use tokio::select;
use tokio::sync::{Mutex, MutexGuard, RwLock};
use tokio::time::Duration;
//use tokio::stream::StreamExt;
//use std::task::{Context, Poll, Waker};
use futures::Stream;
//use libp2p::swarm::SwarmEvent;
//use tokio::sync::MutexGuard;
use futures::task::{Context, Poll, Waker};
use std::pin::Pin;
use tokio::task::yield_now;
use std::future::Future;


use axum::{
    routing::get,
    Router,
    http::StatusCode,
    response::Html,
};

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<i32>>,
	swarm: Arc<Mutex<Swarm<SubfieldBehaviour>>>,
}

async fn increment_counter(State(state): State<AppState>) -> Html<String> {
    let mut counter = state.counter.lock().await;
    *counter += 1;
    Html(format!("Counter: {}", counter))
}

async fn get_counter(State(state): State<AppState>) -> Html<String> {
    let counter = state.counter.lock().await;
    Html(format!("Counter: {}", counter))
}

async fn get_swarm(State(state): State<AppState>) -> Json<Vec<String>> {
	let multiaddrs = state.swarm.lock().await.listeners().into_iter().map(|addr| addr.to_string()).collect::<Vec<_>>();
	
    Json(multiaddrs)
}




#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

	//let keypair = Keypair::generate();

	let swarm = Arc::new(Mutex::new(create_tokio_swarm(SwarmConfig {
		listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".to_string()],
		is_server: true,
		//keypair,
	})?));

	let mut interval = tokio::time::interval(Duration::from_secs(5));

	let swarm_clone = swarm.clone();

    let axum_handle = tokio::spawn(async move {
        // Create shared state
        let state = AppState {
            counter: Arc::new(Mutex::new(0)),
			swarm: swarm_clone,
        };
    
        // Build our application with routes that have access to the shared state
        let app = Router::new()
            .route("/increment", get(increment_counter))
            // Clone the Arc again to use in the second closure
            .route("/get", get(get_counter))
			.route("/bootstrap", get(get_swarm))
            // .layer(TraceLayer::new_for_http()); // Uncomment if TraceLayer is used
			.with_state(state);
	
        // Run it with hyper
        let port = 3000;
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await.unwrap();
        
        tracing::debug!("axum listening on {}", listener.local_addr().unwrap());
        
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });
	
	let swarm_clone = swarm.clone();
	
	let event_handle = tokio::spawn(async move {
		
		loop {
			
			
			let event = {
				let mut lock = swarm_clone.lock().await;
				poll_swarm(&mut lock).await
			};
			

			if let Some(event) = event {
				let mut lock = swarm_clone.lock().await;
				swarm_handle_subfield_event(&mut *lock, event);
			}
			
			
			
			/*
			select! {
				//event = lock.select_next_some() => {
				//	swarm_handle_subfield_event(&mut lock, event)
				//},
				
				_ = interval.tick() => {
					let mut lock = swarm.lock().await;
					let peers = lock.connected_peers().cloned().collect::<Vec<_>>();
					println!("sending pings to peers");
					for peer in peers {
						let message = bys::to_base32(&bys::random(32));
						let _ = lock.behaviour_mut().subfield.send_request(&peer, SubfieldRequest { ping: message.clone() });
						println!("{} sent message to peer {}: {}", lock.local_peer_id(), peer, message);
					}
				},			
				//_ = "" => {}
			}
			*/
			tokio::task::yield_now().await;
		}
	});
	
	Ok(())
	
	/*
			loop {
			
			
			let event = {
				let mut lock = swarm.lock().await;
				poll_swarm(&mut lock).await
			};
			

			if let Some(event) = event {
				let mut lock = swarm.lock().await;
				swarm_handle_subfield_event(&mut *lock, event);
			}
			
			
			
			/*
			select! {
				//event = lock.select_next_some() => {
				//	swarm_handle_subfield_event(&mut lock, event)
				//},
				
				_ = interval.tick() => {
					let mut lock = swarm.lock().await;
					let peers = lock.connected_peers().cloned().collect::<Vec<_>>();
					println!("sending pings to peers");
					for peer in peers {
						let message = bys::to_base32(&bys::random(32));
						let _ = lock.behaviour_mut().subfield.send_request(&peer, SubfieldRequest { ping: message.clone() });
						println!("{} sent message to peer {}: {}", lock.local_peer_id(), peer, message);
					}
				},			
				//_ = "" => {}
			}
			*/
			tokio::task::yield_now().await;
		}*/
}


async fn poll_swarm(swarm: &mut MutexGuard<'_, Swarm<SubfieldBehaviour>>) -> Option<SwarmEvent<SubfieldBehaviourEvent>> {

    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);
    let mut pinned_swarm = Pin::new(swarm);

    match pinned_swarm.poll_next_unpin(&mut cx) {
        Poll::Ready(Some(event)) => Some(event),
        Poll::Ready(None) => None,
        Poll::Pending => {
            yield_now().await;
            None
        }
    }
}