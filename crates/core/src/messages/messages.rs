use crate::u256::*;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub type Key = U256;
pub type Hash = U256;
pub type KeyArr = [u8; 32];
pub type HashArr = [u8; 32];
pub type IdArr = [u8; 32];

/**
 * Action
*/
pub enum Action {
	// System
	Ping = 0,
	Pong = 1,

	// DHT
	NearestNodesByDist = 21,
	NearestNodesByPing = 22,

	// REST
	Post = 40,
	Get = 41,
	Delete = 42,
	Response = 43,

	// Pubsub
	Subscribe = 62,
	Unsubscribe = 63,
	Message = 64,
}

/**
 * Field
*/

type FieldType = Option<U256>;

#[derive(Serialize, Deserialize)]
pub struct Field {
	signer: FieldType,
	cosigner: FieldType,
	tangent: FieldType,
}

impl Field {
	pub fn new(
		signer: FieldType,
		cosigner: FieldType,
		tangent: FieldType,
	) -> Self {
		Self {
			signer,
			cosigner,
			tangent,
		}
	}
}

/**
 * Request/Response
*/
#[derive(Serialize, Deserialize)]
pub struct Request<T> {
	action: T,
	path: Field,
	data: RequestData,
}

impl Request<Action> {
	pub fn new(action: Action, path: Field, data: RequestData) -> Self {
		Self { action, path, data }
	}
}

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
	action: T,
	path: Field,
	data: ResponseData,
}

impl Response<Action> {
	pub fn new(action: Action, path: Field, data: ResponseData) -> Self {
		Self { action, path, data }
	}
}

/**
 * Request/Response Data
*/

#[derive(Serialize, Deserialize)]
pub enum RequestData {
	None,
	Bool(bool),
	Bytes(Bytes),
	Hash(Hash),
	Key(Key),
	Timestamp(chrono::DateTime<chrono::Utc>),
}
pub type ResponseData = RequestData;

pub type Service<I, O> = (Action, Request<I>, Response<O>);
