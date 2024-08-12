#![allow(unused)]
extern crate subfield_macro_proto_ext_proc;
pub use subfield_macro_proto_ext_proc::*;

pub trait ProtoExt {
    // fn to_string(&self) -> String;
    // fn from_string(s: &str) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
    // fn to_json(&self) -> String;
    // fn from_json(s: &str) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
    fn _set_string_cache(&mut self, s: &str);
    fn _get_string_cache(&self) -> &str;
}