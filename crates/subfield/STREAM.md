# TODO
[ ] - 

# How It Works

Terms
	- to dial a node, you need both a PeerId and a Multiaddr
	- Multiaddr exchange must be handled by the protocol
	- Streams are multiplexed over a single actual connection between two peers, and can be created cheaply

behaviour.rs
	Behaviour
	    shared: Arc<Mutex<Shared>>,
    	dial_receiver: mpsc::Receiver<PeerId>,
	NetworkBehaviour
		handle_established_inbound_connections() + handle_established_outbound_connection()
			- when a peer connects, create a new Handler and add it to self.shared
		on_swarm_event()
			- listen to events from the swarm, mostly forward them to self.shared
			- ConnectionEstablished - run shared.on_connection_established
			- ConnectionClosed - run shared.on_connection_closed
			- DialFailure - run shared.on_connection_failed
		on_connection_handler_event()
			- listen to events from the handler (unimplemented)
		poll()
			- poll the dial_receiver for a peerId (cx is used for poll_unpin)
			- if there is one, dial the peer if they are both disconnected and there is currently no ongoing dialing attempt
			- the poll function can either return a ToSwarm event (unimplemented) or send an event to a Connection Handler via ToSwarm::NotifyHandler
			- if there is nothing to be done, return Poll::Pending
		
control.rs
	Control
		shared: Arc<Mutex<Shared>>
		
		open_stream()
			- attempt to open a new Stream for a Protocol and Peer
			- send a NewStream to a Peer's Handler via a Shared::lock().sender(peer)
			- attempt to connect to the Peer if not already connected
			- fail if the Peer does not support the Protocol


handler.rs
	NewStream
		- Message from a Control to a Handler to negotiate a new outbound stream.
		protocol: StreamProtocol
		sender: oneshot::Sender<Result<Stream, OpenStreamError>>
	Handler
		remote: PeerId
			- The remote peer that this Handler instance is for, there is only one handler per peer
		shared: Arc<Mutex<Shared>>
			- A handle to the Shared data
		receiver: Receiver<NewStream>
		pending_upgrade: Option<(
			StreamProtocol,
			oneshot::Sender<Result<Stream, OpenStreamError>>,
		)>
		
		
		

shared.rs
	Shared
		supported_inbound_protocols: HashMap<StreamProtocol, mpsc::Sender<(PeerId, Stream)>>,
			- Tracks the supported inbound protocols created via control.accept
			- For each StreamProtocol, we hold the mpsc::Sender corresponding to the mpsc::Receiver in IncomingStreams
		connections: HashMap<ConnectionId, PeerId>,
    	senders: HashMap<ConnectionId, mpsc::Sender<NewStream>>,
    	pending_channels: HashMap<PeerId, (mpsc::Sender<NewStream>, mpsc::Receiver<NewStream>)>,
			- Tracks channel pairs for a peer whilst we are dialing them.
    	dial_sender: mpsc::Sender<PeerId>,
    		- Sender for peers we want to dial.
    		- We manage this through a channel to avoid locks as part of NetworkBehaviour::poll
		lock()
			- Return a mutex guard to a Shared
		accept()
			- add a protocol to the accepted protocols
			- create a (PeerId, Stream) channel and return the receiver an IncomingStreams(receiver)
		supported_inbound_protocols()
			- list the actively listening protocols
		on_inbound_stream()
			- if a peer sends a Stream, try to forward it to the StreamProtocol's Sender<(PeerId, Stream)>, if available
		on_connection_established()
			- received from Behaviour, insert peerId into self.connections
		on_connection_closed()
			- received from Behaviour, remove peerId from self.connections
		on_dial_failure()
			- received from Behaviour, remove peer from self.pending_channels
			- if there are still pending streams in the receiver, send an error to them
		sender()
			- get a sender into a for a Peer's Handler
			- if there is already a connection open, clone the sender
			- if we are not connected, add the peer to pending_channels, and send the peerId to the self.dial_sender channel, so it can be retrieved from the dial_receiver in the Behaviuor



upgrade.rs
	Upgrade
		supported_protocols : Vec<StreamProtocol>
		protocol_info()
			- returns an iterable of supported protocols