use axum::Router;
use http::{Method, Request, Response};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::time::Duration;
use tower_http::cors::{Any, CorsLayer};

/*
async fn add_security_headers(request: Request, next: Next) -> Response {

	let response = next.run(request).await;
	response.headers.insert(
		"Cross-Origin-Opener-Policy",
		"same-origin".parse().unwrap(),
	);
	response.headers.insert(
		"Cross-Origin-Embedder-Policy",
		"require-corp".parse().unwrap(),
	);
	response.headers
}
*/

pub fn spawn_axum_loop(app: Router, port: i32) -> tokio::task::JoinHandle<()> {
	tokio::spawn(async move {
		let cors = CorsLayer::new()
			// allow `GET` and `POST` when accessing the resource
			.allow_methods([Method::GET, Method::POST])
			// allow requests from any origin
			//.allow_credentials(
			.allow_origin(Any);

		let app = app.layer(cors);
		//.layer(axum::middleware::from_fn(add_security_headers));

		loop {
			let addr = format!("127.0.0.1:{}", port);
			match TcpListener::bind(&addr).await {
				Ok(listener) => {
					println!("axum listening on {}", addr); // Move the success message here
					match axum::serve(
						listener,
						app.into_make_service_with_connect_info::<SocketAddr>(),
					)
					.await
					{
						Ok(_) => {
							break;
						}
						Err(e) => {
							eprintln!("Failed to start server: {}", e);
							break;
						}
					}
				}
				Err(_) => {
					//port += 1; // Try the next port
					//tokio::time::sleep(Duration::from_millis(100)).await;
					eprintln!("Failed to bind to port {}", port);
					break;
				}
			}
		}
	})
}
