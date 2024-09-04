use crate::*;
use std::collections::BTreeMap;

pub struct OrderedMap<K, V> where K: Ord {
    map: BTreeMap<K, V>,
}

impl<K:Ord, V> OrderedMap<K, V> {
    pub fn new() -> Self {
        OrderedMap {
            map: BTreeMap::new(),
        }
    }
	
	pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
	
	pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }
	
	pub fn successor(&self, key: &K) -> Option<&V> {
		self.map.range((std::ops::Bound::Included(key), std::ops::Bound::Unbounded)).next().map(|(_, v)| v)
	}
	
    pub fn predecessor(&self, key: &K) -> Option<&V> {
        self.map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(key))).next_back().map(|(_, v)| v)
    }
	
}

impl<K:Ord, V> IntoIterator for OrderedMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::collections::btree_map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}



#[test]
fn test_ordered_map() {
    let mut map = OrderedMap::new();
    map.insert([1 as u8], "a");
    map.insert([3 as u8], "b");
    map.insert([5 as u8], "c");
	
    let key_to_search = [4 as u8];

    let predecessor = map.predecessor(&key_to_search);
	assert_eq!(predecessor, Some(&"b"));
	
    let successor = map.successor(&key_to_search);
	assert_eq!(successor, Some(&"c"));
}