#![allow(unused)]
use subfield_proto::v256::V256;
use subfield_macro_proto_ext::ProtoExt;
use prost::Message;

fn main() {}

#[test]
fn test_v256() { 
	use rand::Rng;
	let mut rng = rand::thread_rng();
	
	for i in 0..100 {
    let v = V256 {
		v: rng.gen(),
		u0: rng.gen(),
		u1: rng.gen(),
		u2: rng.gen(),
		u3: rng.gen(),
	};
		// println!("proto:{}", v.to_string());
	}
}