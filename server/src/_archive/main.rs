
use anyhow::{Result, Error, anyhow};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State
    },
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};


use std::collections::HashMap;
use tokio::sync::Mutex;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Text, Transact, Update};
use dashmap::{mapref::one::MappedRef, DashMap};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::mpsc::channel;
use tracing::{span, Instrument, Level};
use url::Url;
use y_sweet_core::{
    api_types::{
        validate_doc_name, AuthDocRequest, ClientToken, DocCreationRequest, NewDocResponse,
    },
    auth::Authenticator,
    doc_connection::DocConnection,
    doc_sync::DocWithSyncKv,
	store::{Result as StoreResult, Store, StoreError},
    sync::awareness::Awareness,
};
use async_trait::async_trait;
use std::{
    fs::{create_dir_all, remove_file},
    path::PathBuf,
};


// this struct represents state
// unsure if #[derive(Clone)] is needed for the store
pub struct Server {
    docs: DashMap<String, DocWithSyncKv>,
    store: Option<Arc<Box<dyn Store>>>,
    checkpoint_freq: Duration,
    //authenticator: Option<Authenticator>,
    //url_prefix: Option<Url>,
}

impl Server {
	
	// constructor
	pub async fn new(
        store: Option<Box<dyn Store>>,
        checkpoint_freq: Duration,
        //authenticator: Option<Authenticator>,
        //url_prefix: Option<Url>,
    ) -> Result<Self> {
        Ok(Self {
            docs: DashMap::new(),
            store: store.map(Arc::new),
            checkpoint_freq,
            //authenticator,
            //url_prefix,
        })
    }
	
	// test if the doc exists 
    pub async fn doc_exists(&self, doc_id: &str) -> bool {
        
		// check local storage
		if self.docs.contains_key(doc_id) {
            return true;
		// check remote storage
		/*
		} else if let Some(store) = &self.store {
            store
                .exists(&format!("{}/data.ysweet", doc_id))
                .await
                .unwrap_or_default()
        }
		*/
		} else {
			// doc does not exist
			false
		}
    }
	
	// load a doc from an id, creating a new one if it doesn't exist
    pub async fn load_doc(&self, doc_id: &str) -> Result<()> {
		
		// this channel is used to send a signal to the save loop
        let (send, mut recv) = channel(1024);

		// create a new DocWithSyncKv
        let dwskv = DocWithSyncKv::new(
			doc_id, 
			self.store.clone(), 
			move || {
				// the dirty callback is called when the document is modified
            	send.try_send(()).unwrap();
			}
		)
        .await?;
		
		
        dwskv
            .sync_kv() // creates a clone of the .sync_kv attribute
            .persist() // persists the document to the store 
            .await // wait for the persistence to complete
            .map_err(|e| anyhow!("Error persisting: {:?}", e))?;

		// spawn a new task to save the document periodically
		{
            let sync_kv = dwskv.sync_kv();
            let checkpoint_freq = self.checkpoint_freq;
            let doc_id = doc_id.to_string();
            tokio::spawn(
                async move {
                    // TODO: expedite save on shutdown.
                    let mut last_save = std::time::Instant::now();

                    while let Some(()) = recv.recv().await {
                        tracing::info!("Received dirty signal.");
                        let now = std::time::Instant::now();
                        if now - last_save < checkpoint_freq {
                            let timeout = checkpoint_freq - (now - last_save);
                            tracing::info!(?timeout, "Throttling.");
                            tokio::time::sleep(timeout).await;
                            tracing::info!("Done throttling.");
                        }

                        tracing::info!("Persisting.");
                        sync_kv.persist().await.unwrap();
                        last_save = std::time::Instant::now();
                        tracing::info!("Done persisting.");
                    }

                    tracing::info!("Terminating loop.");
                }
                .instrument(span!(Level::INFO, "save_loop", doc_id=?doc_id)),
            );
        }

        self.docs.insert(doc_id.to_string(), dwskv);
        Ok(())
    }
	
	// create a new doc and load it
    pub async fn create_doc(&self) -> Result<String> {
        let doc_id = nanoid::nanoid!();
        self.load_doc(&doc_id).await?;
        tracing::info!(doc_id=?doc_id, "Created doc");
        Ok(doc_id)
    }
	
	
    pub async fn get_or_create_doc(
        &self,
        doc_id: &str,
    ) -> Result<MappedRef<String, DocWithSyncKv, DocWithSyncKv>> {
        if !self.docs.contains_key(doc_id) {
            tracing::info!(doc_id=?doc_id, "Loading doc");
            self.load_doc(doc_id).await?;
        }

        Ok(self
            .docs
            .get(doc_id)
            .expect("Doc should exist, we just created it.")
            .map(|d| d))
    }
	
	pub async fn serve(self, listener : tokio::net::TcpListener ) -> Result<()>{
     
		println!("Starting to serve...");
		let server_state = Arc::new(self);
	
		let app = Router::new()
			.route("/ws/:doc_id", get(handle_socket_upgrade))
			.with_state(server_state);
	
		println!("Server setup complete, starting to listen for connections...");
		axum::serve(listener, app.into_make_service()).await.unwrap();
		println!("Server stopped serving.");
		
		Ok(())
	}
	
	
}



#[derive(Deserialize)]
struct HandlerParams {
    token: Option<String>,
}


// upgrade the connection to a websocket
async fn handle_socket_upgrade(
    Path(doc_id): Path<String>,
    ws: WebSocketUpgrade,
    State(server_state): State<Arc<Server>>,
   // Query(params): Query<HandlerParams>,
) -> Result<Response, http::status::StatusCode> {

	println!("Got request for doc {}", doc_id);
	
	// TODO use params.token for auth
	
    let dwskv = server_state.get_or_create_doc(&doc_id).await.unwrap();
    let awareness = dwskv.awareness();

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, awareness)))
}


// handle the websocket
async fn handle_socket(socket: WebSocket, awareness: Arc<RwLock<Awareness>>) {
	
	// split the socket into a sink and a stream
    let (mut sink, mut stream) = socket.split();
    // create a channel for sending messages
	// a channel is required because the websocket sink is not cloneable
	let (send, mut recv) = channel(1024);

	
	// spawn a task to send messages to the websocket
    tokio::spawn(async move {
		// when a message is received
        while let Some(msg) = recv.recv().await {
			// send the message to the websocket
            let _ = sink.send(Message::Binary(msg)).await;
        }
    });

	// create a new doc connection
    let connection = DocConnection::new(awareness.clone(), move |bytes| {
        if let Err(e) = send.try_send(bytes.to_vec()) {
            tracing::warn!(?e, "Error sending message");
        }
    });

	
	// when a message is received from the websocket that is binary, 
	// convert it to a vector of bytes and send it to the connection
    while let Some(msg) = stream.next().await {
        let msg = match msg {
            Ok(Message::Binary(bytes)) => bytes,
            Ok(Message::Close(_)) => break,
            Err(_e) => {
                // The stream will complain about things like
                // connections being lost without handshake.
                continue;
            }
            msg => {
                tracing::warn!(?msg, "Received non-binary message");
                continue;
            }
        };

		// send the message to the connection
        if let Err(e) = connection.send(&msg).await {
            tracing::warn!(?e, "Error handling message");
        }
    }
}

pub struct FileSystemStore {
    base_path: PathBuf,
}

impl FileSystemStore {
    pub fn new(base_path: PathBuf) -> std::result::Result<Self, std::io::Error> {
        create_dir_all(base_path.clone())?;
        Ok(Self { base_path })
    }
}

#[async_trait]
impl Store for FileSystemStore {
    async fn init(&self) -> StoreResult<()> {
        Ok(())
    }

    async fn get(&self, key: &str) -> StoreResult<Option<Vec<u8>>> {
        let path = self.base_path.join(key);
        let contents = std::fs::read(path);
        match contents {
            Ok(contents) => Ok(Some(contents)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StoreError::ConnectionError(e.to_string())),
        }
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> StoreResult<()> {
        let path = self.base_path.join(key);
        create_dir_all(path.parent().expect("Bad parent"))
            .map_err(|_| StoreError::NotAuthorized("Error creating directories".to_string()))?;
        std::fs::write(path, value)
            .map_err(|_| StoreError::NotAuthorized("Error writing file.".to_string()))?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> StoreResult<()> {
        let path = self.base_path.join(key);
        remove_file(path)
            .map_err(|_| StoreError::NotAuthorized("Error removing file.".to_string()))?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> StoreResult<bool> {
        let path = self.base_path.join(key);
        Ok(path.exists())
    }
}


#[tokio::main]
async fn main(){
	let server = Server::new(
		Some(Box::new(FileSystemStore::new(PathBuf::from("data")).unwrap())),
		Duration::from_secs(5)
	).await.unwrap();
	
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
	
	let handle = tokio::spawn(async move {
		server.serve(listener).await.unwrap();
	});
	
	tokio::signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C signal handler");

	handle.abort();
}