use crate::yrs_axum::YrsDoc;
use dashmap::DashMap;
use std::sync::Arc;

// the state of the yjs connections
#[derive(Clone)]
pub struct GlobalAppState {
	docs: Arc<DashMap<String, YrsDoc>>,
}

impl GlobalAppState {
	pub fn new() -> Self {
		Self {
			docs: Arc::new(DashMap::new()),
		}
	}

	pub fn get_docs(&self) -> &Arc<DashMap<String, YrsDoc>> {
		&self.docs
	}
}
