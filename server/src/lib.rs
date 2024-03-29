#![feature(portable_simd)]

//folders
pub mod schemas {
	pub use heapswap_schemas::*;
} // auto-generated capnp schemas
pub mod embeddings; // embeddings
pub mod yrs_axum; // axum handlers for yrs/yjs

// files
pub mod api_routers;
pub mod app_state;
pub mod serve;
pub mod sql;

// macros
pub mod macros {
	pub use heapswap_macros::*;
	pub use lazy_static::lazy_static;
	pub use terny::iff;
	pub use timeit::{get_time, timeit, timeit_loops};
}
