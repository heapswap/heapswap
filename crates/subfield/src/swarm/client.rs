use libp2p::request_response::{InboundRequestId, OutboundRequestId};
use libp2p_stream::{Control, IncomingStreams};
use std::sync::atomic::{AtomicU64, Ordering};
pub use crate::*;
use futures::{TryFuture, TryFutureExt};


pub type SubfieldHandle = u64;


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
	store: DashMap<CompleteKey, Record>,
	
	// control
	// incoming_streams: Arc<IncomingStreams>,
	incoming_streams: Arc<Mutex<IncomingStreams>>,
	control: Control,
	
	// keypairs for use in signing requests
	keypairs: DashMap<PublicKey, Keypair>,
	
	// open requests
	open_requests: DashMap<OutboundRequestId, ()>,
	

	// maps a switchboard handle to a set of swarm handles
	splitter: DashMap<SubfieldHandle, [OutboundRequestId; 3]>,
	// maps a swarm handle to a switchboard handle
	unsplitter: DashMap<OutboundRequestId, SubfieldHandle>,
	
	current_handle: Arc<AtomicU64>,
	
}

impl SubfieldClient {
	/*
	Constructor
	*/
	pub async fn new(config: SubfieldConfig) -> Result<Self, SubfieldError> {
		let swarm: ThreadsafeSubfieldSwarm = Arc::new(Mutex::new(
			swarm::create_swarm(config.clone())
				.await
				.map_err(|e| SubfieldError::SwarmError)?,
		));
		
		let swarm_clone = swarm.clone();
		let mut swarm_lock = swarm.lock().await;
		let mut control = swarm_lock.behaviour_mut().subfield.new_control();
		let incoming_streams = Arc::new(Mutex::new(control.accept(SUBFIELD_PROTOCOL).unwrap()));
		// drop(control);
		
		let local_peer_id =
			config.keypair.public_key().to_libp2p_peer_id().unwrap();
		let local_peer_key = config.keypair.public_key().v256().to_key();
		Ok(Self {
			local_peer_id,
			local_peer_key,
			config,
			swarm: swarm_clone,
			control,
			incoming_streams,
			store: DashMap::new(),
			keypairs: DashMap::new(),
			open_requests: DashMap::new(),
			splitter: DashMap::new(),
			unsplitter: DashMap::new(),
			current_handle: Arc::new(AtomicU64::new(0)),
		})
	}

	/*
	Getters
	*/
	pub async fn swarm_lock(&self) -> MutexGuard<swarm::SubfieldSwarm> {
		self.swarm.lock().await
	}

	pub async fn incoming_streams(&self) -> MutexGuard<IncomingStreams> {
	 	self.incoming_streams.lock().await
	}
	
	pub fn control(&self) -> Control {
		self.control.clone()
	}
	
	/*
	Utils
	*/	
	pub fn next_handle(&self) -> u64 {
		self.current_handle.fetch_add(1, Ordering::SeqCst)
	}

	
	pub async fn echo(&self, message: String) -> Result<String, SubfieldError> {
		let message_bytes = message.as_bytes();
		let message_len = message_bytes.len();
		
		let Some(peer_id) = self.closest_local_peer(&RoutingKey::random()).await? else {
			return Err(SubfieldError::SelfIsClosest);
		};
	
		let mut s =	self.control().open_stream(peer_id, SUBFIELD_PROTOCOL).await.map_err(|e| SubfieldError::FailedToOpenStream)?;
		
		s.write_all(&message_bytes).await.map_err(|e| SubfieldError::FailedToWriteStream)?;
		
		let mut buf = vec![0; message_len];
		s.read_exact(&mut buf).await.map_err(|e| SubfieldError::FailedToReadStream)?;
		
		s.close().await.map_err(|e| SubfieldError::FailedToCloseStream)?;
		
		Ok(message)
	}
}
