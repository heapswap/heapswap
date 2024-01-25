use axum::Router;
use heapswap::{api_routers::api_v0_router, app_state::GlobalAppState};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
	// the main router
	let router = Router::new()
		// api v0
		.nest("/api/v0", api_v0_router())
		.with_state(GlobalAppState::new());

	// create the listener
	let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

	// start the server
	println!("Listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, router.into_make_service())
		.await
		.unwrap();
	println!("Exiting!");
}
