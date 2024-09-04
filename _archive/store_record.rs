use crate::*;
use gluesql::{
	core::store::{
		AlterTable, CustomFunction, CustomFunctionMut, Index, IndexMut,
		Metadata, Store, StoreMut,
	},
	prelude::*,
};
use std::sync::Arc;
use thiserror::Error;
use libp2p::kad::{store::{Error as KadStoreError}, Record as KadRecord, ProviderRecord as KadProviderRecord, RecordKey};


/// Trait for types implementing a record store.
///
/// There are two types of records managed by a `RecordStore`:
///
///   1. Regular (value-)records. These records store an arbitrary value
///      associated with a key which is distributed to the closest nodes
///      to the key in the Kademlia DHT as per the standard Kademlia "push-model".
///      These records are subject to re-replication and re-publication as
///      per the standard Kademlia protocol.
///
///   2. Provider records. These records associate the ID of a peer with a key
///      who can supposedly provide the associated value. These records are
///      mere "pointers" to the data which may be followed by contacting these
///      providers to obtain the value. These records are specific to the
///      libp2p Kademlia specification and realise a "pull-model" for distributed
///      content. Just like a regular record, a provider record is distributed
///      to the closest nodes to the key.
///
impl libp2p::kad::store::RecordStore for SubfieldStore {
	
	type RecordsIter<'a>: Iterator<Item = Cow<'a, Record>>
    where
        Self: 'a;
    type ProvidedIter<'a>: Iterator<Item = Cow<'a, ProviderRecord>>
    where
        Self: 'a;
		
	
	/// Gets a record from the store, given its key.
	fn get(&self, k: &RecordKey) -> Option<Cow<'_, KadRecord>> {
		unimplemented!()
	}

	/// Puts a record into the store.
	fn put(&mut self, r: KadRecord) -> Result<(), KadStoreError> {
		unimplemented!()
	}

	/// Removes the record with the given key from the store.
	fn remove(&mut self, k: &RecordKey) {
		unimplemented!()
	}

	/// Gets an iterator over all (value-) records currently stored.
	fn records(&self) -> Self::RecordsIter<'_> {
		unimplemented!()
	}

	/// Adds a provider record to the store.
	///
	/// A record store only needs to store a number of provider records
	/// for a key corresponding to the replication factor and should
	/// store those records whose providers are closest to the key.
	fn add_provider(&mut self, record: KadProviderRecord) -> Result<(), KadStoreError> {
		unimplemented!()
	}

	/// Gets a copy of the stored provider records for the given key.
	fn providers(&self, key: &RecordKey) -> Vec<KadProviderRecord> {
		unimplemented!()
	}

	/// Gets an iterator over all stored provider records for which the
	/// node owning the store is itself the provider.
	fn provided(&self) -> Self::ProvidedIter<'_> {
		unimplemented!()
	}

	/// Removes a provider record from the store.
	fn remove_provider(&mut self, k: &RecordKey, p: &PeerId) {
		unimplemented!()
	}
}