fn main() {}

#[cfg(test)]
mod server_tests {
	use poem::{listener::TcpListener, Server};
	use poem_grpc::{ClientConfig, Request, Response, RouteGrpc, Status};
	use subfield_proto::*;
	
	poem_grpc::include_proto!("helloworld");
 
	struct GreeterService;

	impl Greeter for GreeterService {
		async fn say_hello(
			&self,
			request: Request<HelloRequest>,
		) -> Result<Response<HelloReply>, Status> {
			let reply = HelloReply {
				message: format!("Hello {}!", request.into_inner().name),
			};
			Ok(Response::new(reply))  
		}
	}
	
	struct GoodbyeService;

	impl Goodbye for GoodbyeService {
		async fn say_goodbye(
			&self,
			request: Request<GoodbyeRequest>,
		) -> Result<Response<GoodbyeReply>, Status> {
			let internal_message = GoodbyeReplyInternal {
				message: format!("Goodbye {}!", request.into_inner().name),
			};

			let reply = GoodbyeReply {
				message: proto_serialize(internal_message)?.to_vec(),
			};
			Ok(Response::new(reply))
		}
	}
	
	

	#[tokio::test]
	async fn test_proto_server() -> Result<(), std::io::Error> {
		let route =
			RouteGrpc::new().add_service(GreeterServer::new(GreeterService)).add_service(GoodbyeServer::new(GoodbyeService));
		Server::new(TcpListener::bind("0.0.0.0:3000"))
			.run(route)
			.await
	}
// }

// #[cfg(test)]
// mod client_tests {
	#[tokio::test]
	async fn test_proto_client(
	) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
		
		let _ = tokio::time::sleep(std::time::Duration::from_secs(1)).await;
		
		let client = GreeterClient::new(
			ClientConfig::builder()
				.uri("http://localhost:3000")
				.build()
				.unwrap(),
		);
		let request = Request::new(HelloRequest {
			name: "Poem".into(),
		});
		let response: HelloReply = client.say_hello(request).await?.into_inner();
		assert_eq!(response.message, "Hello Poem!");
		
		let client = GoodbyeClient::new(
			ClientConfig::builder()
				.uri("http://localhost:3000")
				.build()
				.unwrap(),
		);
		let request = Request::new(GoodbyeRequest {
			name: "Poem".into(),
		});
		let response: GoodbyeReply = client.say_goodbye(request).await?.into_inner();

		// Deserialize GoodbyeReplyInternal from bytes
		let internal_message = proto_deserialize::<GoodbyeReplyInternal>(response.message.into())?;
		
		assert_eq!(internal_message.message, "Goodbye Poem!");
		
		
		
		Ok(())
	}
}