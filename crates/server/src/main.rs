//use heapswap_protos::hello;
use poem::{get, handler, listener::TcpListener, web::Path, Route, Server};

#[handler]
fn hello_handler(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().at("/hello/:name", get(hello_handler));

    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let address = std::env::var("ADDRESS").unwrap_or("0.0.0.0".to_string());
    let listening_address = format!("{}:{}", address, port);
    let localhost_address = format!("http://localhost:{}", port);

    println!("Listening on {}", localhost_address);
    Server::new(TcpListener::bind(listening_address))
        .run(app)
        .await
}
