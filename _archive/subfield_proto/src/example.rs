use subfield::bys;
use subfield_protos::snazzy::shirt::*;
use prost::Message;

fn main() {
    let shirt = Shirt {
        color: "blue".to_string(),
        size: shirt::Size::ExtraLarge as i32,
    };

    let encoded = bys::from_proto(&shirt);

    println!("{}", bys::to_base32(&encoded));

    let decoded = Shirt::decode(encoded).unwrap();

    println!("{:?}", decoded);
}
