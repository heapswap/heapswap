use crate::*;

#[derive(Clone, Debug)]
pub struct SubfieldConfig {
	pub keypair: crypto::Keypair,

	pub mode: SubfieldSwarmMode,

	pub bootstrap_urls: Vec<String>,
	// pub bootstrap_multiaddrs: Vec<String>,
	pub bootstrap_multiaddrs: Vec<libp2p::Multiaddr>,
	#[cfg(feature = "server")]
	pub listen_addresses: Vec<String>,

	pub store_path: String,
}

impl Default for SubfieldConfig {
	fn default() -> Self {
		Self {
			keypair: crypto::Keypair::random(),
			mode: SubfieldSwarmMode::Client,
			bootstrap_urls: vec![
				"http://localhost:3000/bootstrap".to_string(),
				"https://heapswap.com/bootstrap".to_string(),
			],
			bootstrap_multiaddrs: vec![],
			#[cfg(feature = "server")]
			listen_addresses: vec![
				// "/ip4/0.0.0.0/tcp/3000/webrtc".to_string(),
				// "/ip6/::/tcp/3000/webrtc".to_string(),
			],
			store_path: String::from("_subfield_store"),
		}
	}
}

impl SubfieldConfig {
	pub async fn get_bootstrap_multiaddrs_from_urls(
		&mut self,
	) -> EResult<Self> {
		// let mut config = self.clone();
		// let mut bootstrap_multiaddrs = vec![];

		for url in self.bootstrap_urls.clone() {
			tracing::info!("Dialing bootstrap URL: {:?}", url);

			// get the json multiaddr list from the url
			let multiaddr_list_res =
				reqwest::get(url).await?.json::<Vec<String>>().await;

			match multiaddr_list_res {
				Ok(multiaddr_list) => {
					for multiaddr in multiaddr_list {
						self.bootstrap_multiaddrs
							.push(multiaddr.parse::<libp2p::Multiaddr>()?);
					}
				}
				Err(e) => {
					// failed to get the multiaddr list, try the next url
				}
			}
		}

		if self.bootstrap_multiaddrs.is_empty() {
			tracing::error!("Failed to get bootstrap multiaddrs from urls");
			// return Err(eyr!("Failed to get bootstrap multiaddrs from urls"));
		}

		Ok(self.clone())
	}
}
