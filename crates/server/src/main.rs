use heapswap_core::bys;
use heapswap_protos::snazzy::shirt;
use prost::Message;

fn main() {
    let shirt = shirt::Shirt {
        color: "blue".to_string(),
        size: shirt::shirt::Size::ExtraLarge as i32,
    };

    let encoded = bys::Bytes::from(shirt.encode_to_vec());

    println!("{}", bys::to_base32(&encoded));

    let decoded = shirt::Shirt::decode(encoded.to_vec().as_slice()).unwrap();

    println!("{:?}", decoded);
}
