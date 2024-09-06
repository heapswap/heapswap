use crate::*;
use std::hash::Hash;

struct SetMap<
	K: Eq + Hash + Clone + std::fmt::Debug,
	V: Eq + Hash + Clone + std::fmt::Debug,
> {
	kv_map: HashMap<K, HashSet<V>>,
	vk_map: HashMap<V, HashSet<K>>,
}

impl<
		K: Eq + Hash + Clone + std::fmt::Debug,
		V: Eq + Hash + Clone + std::fmt::Debug,
	> SetMap<K, V>
{
	pub fn new() -> Self {
		Self {
			kv_map: HashMap::new(),
			vk_map: HashMap::new(),
		}
	}

	pub fn insert(&mut self, k: K, v: V) {
		self.kv_map
			.entry(k.clone())
			.or_insert_with(HashSet::new)
			.insert(v.clone());
		self.vk_map.entry(v).or_insert_with(HashSet::new).insert(k);
	}

	pub fn remove_only(&mut self, k: &K, v: &V) -> bool {
		let mut exists = false;

		if let Some(v_set) = self.kv_map.get_mut(k) {
			exists = true;
			v_set.remove(&v);
			if v_set.is_empty() {
				self.kv_map.remove(k);
			}
		}

		if let Some(k_set) = self.vk_map.get_mut(v) {
			exists = true;
			k_set.remove(&k);
			if k_set.is_empty() {
				self.vk_map.remove(v);
			}
		}

		exists
	}

	// remove a key from the kv_map and remove all occurences of k from the vk_map
	// returns the values that were removed
	pub fn remove_key(&mut self, k: &K) -> HashSet<V> {
		// remove the key from the kv_map
		let v_set = self.kv_map.remove(k).unwrap_or_default();

		// loop through all values in the v_set and remove the key from the vk_map
		for v in v_set.iter() {
			if let Some(k_set) = self.vk_map.get_mut(v) {
				k_set.remove(k);
				if k_set.is_empty() {
					self.vk_map.remove(&v);
				}
			}
		}
		v_set
	}

	// remove a value from the vk_map and remove all occurences of v from the kv_map
	// returns the keys that were removed
	pub fn remove_value(&mut self, v: &V) -> HashSet<K> {
		// remove the value from the vk_map
		let k_set = self.vk_map.remove(v).unwrap_or_default();

		// loop through all keys in the k_set and remove the value from the kv_map
		for k in k_set.iter() {
			if let Some(v_set) = self.kv_map.get_mut(k) {
				v_set.remove(v);
				if v_set.is_empty() {
					self.kv_map.remove(k);
				}
			}
		}
		k_set
	}

	pub fn get_key(&mut self, k: &K) -> Option<HashSet<V>> {
		self.kv_map.get(k).map(|ref_val| ref_val.clone())
	}

	pub fn get_value(&mut self, v: &V) -> Option<HashSet<K>> {
		self.vk_map.get(v).map(|ref_val| ref_val.clone())
	}
}

#[test]
fn test_setmap() {
	let mut sm = SetMap::new();

	// kv
	// a -> z, y
	// b -> y
	// c -> y

	// vk
	// z -> a
	// y -> a, b, c

	sm.insert("a".to_string(), "z".to_string());
	sm.insert("a".to_string(), "y".to_string());
	sm.insert("b".to_string(), "y".to_string());
	sm.insert("b".to_string(), "y".to_string());
	sm.insert("c".to_string(), "y".to_string());

	// test keys

	let Some(a_vals) = sm.get_key(&"a".to_string()) else {
		panic!("a_vals not found");
	};
	assert!(a_vals == HashSet::from_iter(["z".to_string(), "y".to_string()]));

	let Some(b_vals) = sm.get_key(&"b".to_string()) else {
		panic!("b_vals not found");
	};
	assert!(b_vals == HashSet::from_iter(["y".to_string()]));

	let Some(c_vals) = sm.get_key(&"c".to_string()) else {
		panic!("c_vals not found");
	};
	assert!(c_vals == HashSet::from_iter(["y".to_string()]));

	// test values

	let Some(z_keys) = sm.get_value(&"z".to_string()) else {
		panic!("z_keys not found");
	};
	assert!(z_keys == HashSet::from_iter(["a".to_string()]));

	let Some(y_keys) = sm.get_value(&"y".to_string()) else {
		panic!("y_keys not found");
	};
	assert!(
		y_keys
			== HashSet::from_iter([
				"a".to_string(),
				"b".to_string(),
				"c".to_string()
			])
	);

	// remove a key
	sm.remove_key(&"b".to_string());
	// kv
	// a -> z, y
	// b -> none
	// c -> y

	// vk
	// z -> a
	// y -> a, c

	let b_vals = sm.get_key(&"b".to_string());
	assert!(b_vals == None);

	let Some(y_keys) = sm.get_value(&"y".to_string()) else {
		panic!("y_keys not found");
	};
	assert!(y_keys == HashSet::from_iter(["a".to_string(), "c".to_string()]));

	// remove only
	sm.remove_only(&"a".to_string(), &"z".to_string());
	// kv
	// a -> y
	// b -> none
	// c -> y

	// vk
	// z -> none
	// y -> a, c

	let Some(a_vals) = sm.get_key(&"a".to_string()) else {
		panic!("a_vals not found");
	};
	assert!(a_vals == HashSet::from_iter(["y".to_string()]));

	let z_keys = sm.get_value(&"z".to_string());
	assert!(z_keys == None);
}
