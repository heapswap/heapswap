use std::sync::Arc;
use tokio::sync::RwLock;
use y_sync::awareness::Awareness;
use y_sync::net::BroadcastGroup;
use yrs::Doc;

pub type AwarenessRef = Arc<RwLock<Awareness>>;
pub type BroadcastRef = Arc<BroadcastGroup>;

// ignore dead code warning
#[allow(dead_code)]
pub struct YrsDoc {
	awareness: AwarenessRef,
	broadcast: BroadcastRef,
}

// YrsDoc is a wrapper around a Yrs Doc that also contains
// an awareness and broadcast group
impl YrsDoc {
	pub async fn new() -> Self {
		// create the awareness
		let awareness: AwarenessRef = {
			let doc = Doc::new();
			// create the awareness
			Arc::new(RwLock::new(Awareness::new(doc.clone())))
		};
		let broadcast =
			Arc::new(BroadcastGroup::new(awareness.clone(), 32).await);

		Self {
			awareness,
			broadcast,
		}
	}

	pub fn get_awareness(&self) -> AwarenessRef {
		self.awareness
			.clone()
	}

	pub fn get_broadcast(&self) -> BroadcastRef {
		self.broadcast
			.clone()
	}
}
