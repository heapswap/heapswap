use crate::*;


#[test]
fn test_subkey_combinations() {
	// all combinations 2^3 - 1 = 7
	let subkey = Subkey::new(Some(V256::random256()), Some(V256::random256()), Some(V256::random256()));
	let hashes = subkey.all_hashes().unwrap();
	assert_eq!(hashes.len(), 7);
	
	// test subkey as used for keys in maps
	let mut map = HashMap::new();
	for hash in hashes.clone() {
		map.insert(hash.clone(), hash.clone());
	}
	// get a random hash from the map
	let hash = map.get(&hashes[0]).unwrap();
	assert_eq!(hash, &hashes[0]);
}