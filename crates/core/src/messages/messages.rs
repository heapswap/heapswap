use crate::u256::*;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub type Key = U256;
pub type Hash = U256;
pub type KeyArr = [u8; 32];
pub type HashArr = [u8; 32];
pub type IdArr = [u8; 32];

pub enum Action {
    // REST
    Post = 0,
    Get = 1,
    Delete = 2,
    Response = 3,

    // Pubsub
    Subscribe = 32,
    Unsubscribe = 33,
    Message = 34,
}

#[derive(Serialize, Deserialize)]
pub struct Field {
    signer: String,
    cosigner: String,
    tangent: String,
}

impl Field {
    pub fn new(signer: String, cosigner: String, tangent: String) -> Self {
        Self {
            signer,
            cosigner,
            tangent,
        }
    }
}

pub struct Request {
    id: IdArr,
    action: Action,
    path: Field,
    data: Bytes,
}

pub struct Response {
    id: IdArr,
    action: Action,
    path: Field,
    data: Bytes,
}
