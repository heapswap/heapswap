#![allow(unreachable_code, unreachable_patterns, unreachable_pub)]
use crate::crypto::*;
use crate::subfield::*;
use crate::swarm::*;
use async_std::task::sleep;
use bevy::{app::PanicHandlerPlugin, log::LogPlugin, prelude::*};
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
//use futures::StreamExt;
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;
use bevy::tasks::futures_lite::StreamExt;

/// An async task that takes time to compute!
async fn long_task() -> u32 {
	sleep(Duration::from_millis(1000)).await;
	5
}
use async_std::future;

#[cfg(target_arch = "wasm32")]
async fn start_libp2p_swarm_wasm() -> SubfieldSwarm {
	let keypair = keys::Keypair::random();

	// use reqwest to get list of multaddrs (json) from localhost:3000
	//let bootstrap_multiaddrs =
	//	reqwest::get("http://localhost:3000/bootstrap")
	//		.await
	//		.unwrap()
	//		.json::<Vec<String>>()
	//		.await
	//		.unwrap();

	// use reqwest to get list of multaddrs (json) from localhost:3000
	let mut bootstrap_multiaddrs = None;
	while bootstrap_multiaddrs.is_none() {
		
		//async_std::task::block_on(async {
			let res = reqwest::get("http://localhost:3000/bootstrap").await;
			if res.is_ok() {
				bootstrap_multiaddrs =
					Some(res.unwrap().json::<Vec<String>>().await.unwrap());
			}
		//});	
	}
	let bootstrap_multiaddrs = bootstrap_multiaddrs.unwrap();

	tracing::info!("Bootstrap Multiaddrs: {:?}", bootstrap_multiaddrs);

	// create a libp2p swarm
	let swarm_config = SubfieldSwarmConfig {
		keypair,
		bootstrap_multiaddrs,
		listen_addresses: vec![],
	};

	//let mut swarm = swarm_create(swarm_config).await.unwrap();
	
	//let mut swarm;
	//async_std::task::block_on(async {
	let swarm = swarm_create(swarm_config).await.unwrap();
	//});

	return swarm;
}

#[cfg(target_arch = "wasm32")]
fn libp2p_system(mut task_runner: AsyncTaskRunner<()>) {
	use bevy::tasks::futures_lite::StreamExt;

	match task_runner.poll() {
		AsyncTaskStatus::Idle => {
			task_runner.start(async move {
				let _ = start_libp2p_swarm_wasm().await;
			});
			info!("Started Libp2p Swarm");
		}
		AsyncTaskStatus::Pending => {}
		AsyncTaskStatus::Finished(_) => {}
	}
}

#[cfg(target_arch = "wasm32")]
pub async fn main(using_bevy: bool) {
	
	if using_bevy{
	
	let mut app = App::new();
	app.add_plugins((
		MinimalPlugins,
		LogPlugin {
			level: tracing::Level::DEBUG,
			..default()
		},
		PanicHandlerPlugin,
	));

	#[cfg(target_arch = "wasm32")]
	app.add_systems(Update, libp2p_system);

	app.run();

	} else {
		//#[cfg(target_arch = "wasm32")]	
		let mut swarm = start_libp2p_swarm_wasm().await;
		//});
		
		
		loop {
			match swarm.next().await {
				Some(event) => {
					//tracing::info!("Event: {:?}", event);
				}
				None => {}
			}
		}
	}
}
