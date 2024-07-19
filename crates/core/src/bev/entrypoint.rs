use crate::crypto::*;
use crate::subfield::*;
use crate::swarm::*;
use async_std::task::sleep;
use bevy::{app::PanicHandlerPlugin, log::LogPlugin, prelude::*};
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use futures::StreamExt;
use std::time::Duration;

/// An async task that takes time to compute!
async fn long_task() -> u32 {
	sleep(Duration::from_millis(1000)).await;
	5
}
use async_std::future;

#[cfg(target_arch = "wasm32")]
fn libp2p_system(mut task_runner: AsyncTaskRunner<()>) {
	use bevy::tasks::futures_lite::StreamExt;

	match task_runner.poll() {
		AsyncTaskStatus::Idle => {
			task_runner.start(async move {
				let keypair = keys::Keypair::random();

				// use reqwest to get list of multaddrs (json) from localhost:3000
				let bootstrap_multiaddrs =
					reqwest::get("http://localhost:3000/bootstrap")
						.await
						.unwrap()
						.json::<Vec<String>>()
						.await
						.unwrap();

				tracing::info!(
					"Bootstrap Multiaddrs: {:?}",
					bootstrap_multiaddrs
				);

				// create a libp2p swarm
				let swarm_config = SwarmConfig {
					keypair,
					bootstrap_multiaddrs,
                    listen_addresses: vec![],
				};

				let mut swarm = swarm_create(swarm_config).await.unwrap();

				loop {
					match futures::StreamExt::next(&mut swarm).await {
						Some(event) => {
							let _ = swarm_handle_event(&mut swarm, event).await;
						}
						None => {}
					}
				}
			});
			info!("Started Libp2p Swarm");
		}
		AsyncTaskStatus::Pending => {}
		AsyncTaskStatus::Finished(_) => {}
	}
}

pub fn main() {
	//let mut app = App::new();

	//app.add_plugins((MinimalPlugins, LogPlugin::default(), PanicHandlerPlugin));

	//#[cfg(target_arch = "wasm32")]
	//app.add_systems(Update, libp2p_system);

	//app.run();

	let swarm = wasm_bindgen_futures::spawn_local(async {
		let keypair = keys::Keypair::random();

		// use reqwest to get list of multaddrs (json) from localhost:3000
		let bootstrap_multiaddrs =
			reqwest::get("http://localhost:3000/bootstrap")
				.await
				.unwrap()
				.json::<Vec<String>>()
				.await
				.unwrap();

		tracing::info!("Bootstrap Multiaddrs: {:?}", bootstrap_multiaddrs);

		// create a libp2p swarm
		let swarm_config = SwarmConfig {
			keypair,
			bootstrap_multiaddrs,
			listen_addresses: vec![]
		};

		let mut swarm = swarm_create(swarm_config).await.unwrap();

		loop {
			match futures::StreamExt::next(&mut swarm).await {
				Some(event) => {
					let _ = swarm_handle_event(&mut swarm, event).await;
				}
				None => {}
			}
		}
	});

	//async_std::task::block_on(swarm);
}
