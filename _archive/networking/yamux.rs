// Dependencies in Cargo.toml
// yamux = "0.8.1"
//// tokio = { version = "1", features = ["full"] }

use yamux::Config;
use std::error::Error;
//#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let addr = "127.0.0.1:12345";
	////tokio::spawn(async move {
	//	server(addr).await;
	//});

	////tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Wait for the server to start

	client(addr).await?;

	Ok(())
}

async fn server(addr: &str) {
	let listener = TcpListener::bind(addr).await.unwrap();
	println!("Server listening on {}", addr);

	while let Ok((stream, _)) = listener.accept().await {
		//tokio::spawn(handle_connection(stream));
	}
}

async fn handle_connection(stream: TcpStream) {
	let mut connection = yamux::Connection::new(stream, Config::default(), yamux::Mode::Server);

	while let Ok(Some(mut stream)) = connection.accept().await {
		//tokio::spawn(async move {
		//	let mut buf = [0; 1024];
		//	while let Ok(n) = stream.read(&mut buf).await {
		//		if n == 0 { break; }
		//		println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
		//	}
		//});
	}
}

async fn client(addr: &str) -> Result<(), Box<dyn Error>> {
	let stream = TcpStream::connect(addr).await?;
	let mut connection = yamux::Connection::new(stream, Config::default(), yamux::Mode::Client);

	let mut stream = connection.open_stream().await?.unwrap();
	stream.write_all(b"Hello, yamux!").await?;
	stream.flush().await?;

	Ok(())
}
