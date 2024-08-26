pub use crate::*;
// pub use subfield_proto::*;


#[test]
fn test_alias_proto() {
	let mut bytes = vec![1, 2, 3];
	let mut versioned_bytes = subfield_proto::VersionedBytes{
		version: 1,
		data: bytes.clone(),
	};
	
	let public_key = subfield_proto::PublicKey{
		version: 1,
		data: bytes,
	};

	let serialized = proto::serialize(public_key.clone()).unwrap();
	let deserialized = proto::deserialize::<subfield_proto::PublicKey>(serialized).unwrap();

	assert_eq!(deserialized, public_key);
}

#[test]
fn test_proto_versioned_bytes() {	
	let vec256 = VersionedBytes::random256();
	let vec256_serialized = vec256.to_proto_bytes().unwrap();
	let vec256_deserialized = VersionedBytes::from_proto_bytes(vec256_serialized).unwrap();
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
	let deserialized = crypto::PrivateKey::from_proto_bytes(serialized).unwrap();
	assert_eq!(private_key, deserialized);
}

#[test]
fn test_proto_keypair() {
	let keypair = crypto::Keypair::random();
	let keypair_proto = keypair.to_proto().unwrap();
	let keypair_from_proto = crypto::Keypair::from_proto(keypair_proto).unwrap();
	assert_eq!(keypair, keypair_from_proto);
}