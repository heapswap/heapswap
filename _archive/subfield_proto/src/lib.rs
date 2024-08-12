#![allow(unused)]
// extern crate serde;
// extern crate serde_json;
extern crate once_cell;
extern crate subfield_macro_proto_ext;
extern crate prost;
use crate::prost::Message;

include!(concat!(env!("OUT_DIR"), "/lib.rs"));
