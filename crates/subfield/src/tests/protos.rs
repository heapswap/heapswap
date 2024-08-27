pub use crate::*;
// pub use subfield_proto::*;

#[test]
fn test_alias_proto() {
	let mut bytes = vec![1, 2, 3];
	let mut versioned_bytes = subfield_proto::VersionedBytes {
		version: 1,
		data: bytes.clone(),
	}; 

	let public_key = subfield_proto::PublicKey {
		version: 1,
		data: bytes,
	};

	let serialized = proto::serialize(&public_key).unwrap();
	let deserialized =
		proto::deserialize::<subfield_proto::PublicKey>(serialized).unwrap();

	assert_eq!(deserialized, public_key);
}

#[test]
fn test_proto_versioned_bytes() {
	let vec256 = VersionedBytes::random256();
	let vec256_serialized = vec256.to_proto_bytes().unwrap();
	let vec256_deserialized =
		VersionedBytes::from_proto_bytes(vec256_serialized).unwrap();
	assert_eq!(vec256, vec256_deserialized);
}

#[test]
fn test_proto_public_key() {
	let public_key = crypto::PublicKey::new(V256::random256());
	let serialized = public_key.to_proto_bytes().unwrap();
	let deserialized = crypto::PublicKey::from_proto_bytes(serialized).unwrap();
	assert_eq!(public_key, deserialized);
}

#[test]
fn test_proto_private_key() {
	let private_key = crypto::PrivateKey::new(V256::random256());
	let serialized = private_key.to_proto_bytes().unwrap();
	let deserialized =
		crypto::PrivateKey::from_proto_bytes(serialized).unwrap();
	assert_eq!(private_key, deserialized);
}

#[test]
fn test_proto_keypair() {
	let keypair = crypto::Keypair::random();
	let keypair_proto = keypair.to_proto().unwrap();
	let keypair_from_proto =
		crypto::Keypair::from_proto(keypair_proto).unwrap();
	assert_eq!(keypair, keypair_from_proto);
}

#[test]
fn test_proto_subkey() {
	let subkey = protocol::Subkey::random();
	let subkey_proto = subkey.to_proto().unwrap();
	let subkey_from_proto = protocol::Subkey::from_proto(subkey_proto).unwrap();
	assert_eq!(subkey, subkey_from_proto);
}

#[test]
fn test_proto_timestamp() {
	let timestamp = protocol::now_timestamp_proto();
	let datetime = protocol::timestamp_proto_to_datetime(timestamp).unwrap();
	let timestamp_from_datetime =
		protocol::datetime_to_timestamp_proto(datetime);
	assert_eq!(timestamp, timestamp_from_datetime);
}


#[test]
fn test_proto_request_serialization() {
	
	let timestamp = protocol::now_timestamp_proto();
	
	let request = protocol::SubfieldRequest::new(protocol::SubfieldRequestType::Ping(
		proto::PingRequest {
			timestamp: Some(timestamp.clone()),
		}
	));
	
	let proto_serialized = proto::serialize(request.proto()).unwrap();
	let proto_deserialized = proto::deserialize::<proto::SubfieldRequest>(proto_serialized).unwrap();
	
	assert_eq!(request.proto(), &proto_deserialized);
	
	let bincode_serialized = bincode::serialize(&request).unwrap();
	let bincode_deserialized = bincode::deserialize::<protocol::SubfieldRequest>(&bincode_serialized).unwrap();
	
	assert_eq!(request.proto(), bincode_deserialized.proto());
	
}