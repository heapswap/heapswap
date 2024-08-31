use crate::*;

#[test]
fn test_key_hash_combinations() {
	// all combinations 2^3 - 1 = 7
	let key = PartialKey {
		signer: Some(V256::random256()),
		cosigner: Some(V256::random256()),
		tangent: Some(V256::random256()),
	};
	let hashes = key.hash_combinations().unwrap();
	assert_eq!(hashes.len(), 7);

	// all combinations 2^2 - 1 = 3
	let key = PartialKey {
		signer: Some(V256::random256()),
		cosigner: Some(V256::random256()),
		tangent: None,
	};
	let hashes = key.hash_combinations().unwrap();
	assert_eq!(hashes.len(), 3);

	// all combinations 2^1 - 1 = 1
	let key = PartialKey {
		signer: Some(V256::random256()),
		cosigner: None,
		tangent: None,
	};
	let hashes = key.hash_combinations().unwrap();
	assert_eq!(hashes.len(), 1);

	// test key as used for keys in maps
	let mut map = HashMap::new();
	for hash in hashes.clone() {
		map.insert(hash.clone(), hash.clone());
	}
	// get a random hash from the map
	let hash = map.get(&hashes[0]).unwrap();
	assert_eq!(hash, &hashes[0]);
}
