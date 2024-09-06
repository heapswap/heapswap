pub use crate::*;
use libp2p_stream::{Control, IncomingStreams};
use std::sync::atomic::{AtomicU64, Ordering};

pub type SubfieldHandle = u64;

#[derive(Getters)]
pub struct SwarmManager {
	
	#[getset(get = "pub")]
	local_peer_id: libp2p::PeerId,
	#[getset(get = "pub")]
	local_peer_record_key: libp2p::kad::RecordKey,
	#[getset(get = "pub")]
	local_peer_kbucket_key: libp2p::kad::KBucketKey<Vec<u8>>,

	#[getset(get = "pub")]
	config: SubfieldConfig,

	swarm: ThreadsafeSubfieldSwarm,
	// store: store::SubfieldStore,
	#[getset(get = "pub")]
	store: DashMap<CompleteKey, Record>,

	// streams
	incoming_streams: Arc<Mutex<IncomingStreams>>,
	control: Control,

	
	// kad
	#[getset(get = "pub")]
	kad_queries: DashMap<libp2p::kad::QueryId, UnboundedSender<PeerId>>,
	

	current_handle: Arc<AtomicU64>,
}

impl SwarmManager {
	/*
	Constructor
	*/
	pub async fn new(config: SubfieldConfig) -> Result<Self, SubfieldError> {
		let swarm: ThreadsafeSubfieldSwarm = Arc::new(Mutex::new(
			create_swarm(config.clone())
				.await
				.map_err(|e| SubfieldError::SwarmError)?,
		));

		let swarm_clone = swarm.clone();
		let mut swarm_lock = swarm.lock().await;
		let mut control = swarm_lock.behaviour_mut().subfield.new_control();
		let incoming_streams =
			Arc::new(Mutex::new(control.accept(SUBFIELD_PROTOCOL).unwrap()));

		let local_peer_id =
			config.keypair.public_key().to_libp2p_peer_id().unwrap();
		let local_peer_record_key = config.keypair.public_key().v256().to_record_key();
		let local_peer_kbucket_key = config.keypair.public_key().v256().to_kbucket_key();
		Ok(Self {
			local_peer_id,
			local_peer_record_key,
			local_peer_kbucket_key,
			config,
			swarm: swarm_clone,
			control,
			incoming_streams,
			store: DashMap::new(),
			kad_queries: DashMap::new(),
			current_handle: Arc::new(AtomicU64::new(0)),
		})
	}

	/*
	Getters
	*/
	pub async fn swarm_lock(&self) -> MutexGuard<SubfieldSwarm> {
		self.swarm.lock().await
	}

	pub async fn incoming_streams(&self) -> MutexGuard<IncomingStreams> {
		self.incoming_streams.lock().await
	}

	pub fn control(&self) -> Control {
		self.control.clone()
	}
}
