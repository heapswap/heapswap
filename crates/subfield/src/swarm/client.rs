use libp2p::request_response::{InboundRequestId, OutboundRequestId};

pub use crate::*;
use futures::{TryFuture, TryFutureExt};


#[derive(Clone, Getters)]
pub struct SubfieldClient {
	
	#[getset(get = "pub")]
	local_peer_id: libp2p::PeerId,
	#[getset(get = "pub")]
	local_peer_key: libp2p::kad::KBucketKey<Vec<u8>>,
	
	#[getset(get = "pub")]
	config: SubfieldConfig,

	swarm: swarm::ThreadsafeSubfieldSwarm,
	// store: store::SubfieldStore,
	#[getset(get = "pub")]
	store: DashMap<CompleteSubkey, Record>,
	
	request_handles: Arc<SubfieldRequestHandles>,
}

type HandleMap<T> = DashMap<OutboundRequestId, Sender<T>>;

enum SubfieldRequestRPC {
	Ping(PingRequest),
	Echo(EchoRequest),
	GetRecord(GetRecordRequest),
	DeleteRecord(DeleteRecordRequest),
	PutRecord(PutRecordRequest),
	Subscribe(SubscribeRequest),
	Unsubscribe(UnsubscribeRequest),
}

enum SubfieldResponseRPC {
	Ping(PingResponse),
	Echo(EchoResponse),
	GetRecord(GetRecordResponse),
	DeleteRecord(DeleteRecordResponse),
	PutRecord(PutRecordResponse),
	Subscribe(Receiver<SubscribeResponse>),
	Unsubscribe(UnsubscribeResponse),
}

struct SubfieldRequestHandles {
	// pub kad: PortalManager<libp2p::PeerId>,
	pub subfield: PortalManager<SubfieldResponse>,
}

impl SubfieldRequestHandles {
	pub fn new() -> Self {
		Self {
			// kad: PortalManager::new(),
			subfield: PortalManager::new(),
		}
	}
}

fn get_outbound_request_id(request_id: OutboundRequestId) -> u64 {
	unsafe { std::mem::transmute::<OutboundRequestId, u64>(request_id) }
}

fn get_inbound_request_id(request_id: InboundRequestId) -> u64 {
	unsafe { std::mem::transmute::<InboundRequestId, u64>(request_id) }
}

pub enum LocalClosestKeyResult {
	Ok(libp2p::PeerId),
	// NoPeersConnected,
	SelfIsClosest,
}

impl SubfieldClient {
	/**
	 * Constructor
		*/
	pub async fn new(config: SubfieldConfig) -> EResult<Self> {
		let swarm: ThreadsafeSubfieldSwarm =
			Arc::new(Mutex::new(swarm::create_swarm(config.clone()).await?));
		let request_handles: Arc<SubfieldRequestHandles> =
			Arc::new(SubfieldRequestHandles::new());
		// let store = store::SubfieldStore::new(config.clone()).await?;
		// let peer_id = libp2p::PeerId::from_public_key(libp2p::identity::PublicKey::config.keypair.public_key());
		let local_peer_id =
			config.keypair.public_key().to_libp2p_peer_id().unwrap();
		let local_peer_key = config.keypair.public_key().v256().to_key();
		Ok(Self {
			local_peer_id,
			local_peer_key,
			config,
			swarm,
			store: DashMap::new(),
			request_handles,
		})
	}

	/**
	 * Getters
		*/
	pub async fn swarm(&self) -> MutexGuard<swarm::SubfieldSwarm> {
		self.swarm.lock().await
	}

	// pub async fn cache(
	// 	&self,
	// ) -> MutexGuard<Pin<Box<dyn store::FullStore + Send>>> {
	// 	self.store.cache().await
	// }

	// pub async fn perma(
	// 	&self,
	// ) -> MutexGuard<Pin<Box<dyn store::FullStore + Send>>> {
	// 	self.store.perma().await
	// }
	
	/**
	 * Bootstrap
	*/

	
	/**
	 * Channel Creation
	*/
	
	fn create_portal(&self, handle: u64, request: &SubfieldRequest) -> Result<(), SubfieldError> {
		if request.is_oneshot() {
			let _ = self
			.request_handles
			.subfield
			.create_oneshot_with_handle(&handle);
		} else {
			let _ = self
			.request_handles
			.subfield
			.create_stream_with_handle(&handle);
		}
		Ok(())
	}
	
	pub fn send_response_to_portal(&self, handle: u64, response: SubfieldResponse) -> Result<(), SubfieldError> {
		if response.is_oneshot() {
			let _ = self
			.request_handles
			.subfield
			.send_oneshot(&handle, response);
		} else {
			let _ = self
			.request_handles
			.subfield
			.send_stream(&handle, response);
		}
		Ok(())
	}
	
	pub async fn recv_next_response_from_portal(&self, handle: u64) -> Result<SubfieldResponse, SubfieldError> {
		let handle_is_oneshot = self.request_handles.subfield.handle_is_oneshot(&handle);
		if handle_is_oneshot {
			let response = self.request_handles.subfield.recv_oneshot(&handle).await.map_err(|e| SubfieldError::PortalError(e))?;
			Ok(response)
		} else {
			let response = self.request_handles.subfield.recv_stream(&handle).await.map_err(|e| SubfieldError::PortalError(e))?;
			Ok(response)
		}
	}
	
	// oneshot portals are deleted automatically
	pub fn delete_portal(&self, handle: u64) -> Result<(), SubfieldError> {
		let _ = self.request_handles.subfield.delete_stream(&handle);
		Ok(())
	}
	
	
		
}
