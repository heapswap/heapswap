use crate::*;


#[async_trait]
impl SubfieldServiceTrait for SubfieldClient {


	async fn echo(
		&self,
		subkey: RoutingSubkey,
		message: &str,
	) -> EchoResponse {
		
		let request = SubfieldRequest {
			subkey: subkey.clone(),
			body: SubfieldRequestBody::Echo(EchoRequest {
				message: message.to_string(),
			}),
		};	
		
		let handle = self
			.send_request_to_closest_local_peer(subkey, SubfieldRequest::Echo(request))
			.await.map_err(|e| EchoFailure::ServiceError(e))?;

		// await the response
		match self.recv_next_response_from_portal(handle).await {
			Ok(res) => {
				
				// close the stream
				let _ = self.delete_portal(handle);
				
				let SubfieldResponse::Echo(res) = res else {
					return Err(EchoFailure::ServiceError(SubfieldError::UnexpectedResponseType));
				};

				res
			}
			Err(e) => {
				tracing::error!("Failed to receive echo response: {:?}", e);
				Err(EchoFailure::ServiceError(e))
			}
		}
		
	}
	
	async fn put_record(
		&self,
		subkey: CompleteSubkey,
		record: Record,
	) -> PutRecordResponse {
		self.put_record_with_keypair(subkey, record, self.config().keypair.clone()).await
	}
	
	async fn put_record_with_keypair(
		&self,
		subkey: CompleteSubkey,
		record: Record,
		keypair: Keypair,
	) -> PutRecordResponse {
		
		let requests = record.to_put_record_requests(&subkey, &keypair).map_err(|e| PutRecordFailure::RecordError(e))?;
			
		// send each of the requests to their closest local peers
		// each handle returns a stream channel (impl Future)
		let channels = requests.map(|request| {
			let future = async move {
				let handle = self.send_request_to_closest_local_peer(
					request.routing_subkey.clone(),
					SubfieldRequest::PutRecord(request)
				).await?;

				let res = self.recv_next_response_from_portal(handle).await?;
				Ok(res)
			};
			Box::pin(future)
		});
			
		// wait for all of the channels to return a response
		let responses: Vec<Result<SubfieldResponse, SubfieldError>> = futures::future::join_all(channels).await;
			
		// if at least one of the responses is ok, return the first one
		for response in responses.clone() {
			if response.is_ok() {
				match response.unwrap() {
					SubfieldResponse::PutRecord(res) => {
						return res.clone();
					}
					_ => {
					}
				}
			}
		}
		return Err(PutRecordFailure::ServiceError(SubfieldError::RequestFailed));
	}
	

	/*
	async fn get_record(
		&self,
		subkey: Subkey,
	) -> Result<GetRecordResponse, SubfieldError> {
		
		// let requests = subkey.to_get_record_requests().map_err(|_| SubfieldError::IncompleteSubkey)?;
		
		// // send each of the requests to their closest local peers
		// // each handle returns a stream channel (impl Future)
		// let channels = requests.map(|request| {
		// 	let future = async move {
		// 		let handle = self.send_request_to_closest_local_peer(SubfieldRequest::GetRecord(request)).await?;
		// 		let channel = self.request_handles.subfield.recv_stream(handle);
		// 		channel.await.map_err(|_| SubfieldError::UnexpectedSubfieldResponse)
		// 	};
		// 	Box::pin(future)
		// });
		
		// // wait for any of the channels to return a response
		// let responses = futures::future::select_ok(channels).await?;
		
		// match responses.0.proto_type() {
		// 	SubfieldResponse::GetRecord(res) => {
		// 		Ok(res.clone())
		// 	}
		// 	_ => {
		// 		Err(SubfieldError::UnexpectedSubfieldResponse)
		// 	}
		// }
		
		todo!()
	}

	*/
	
	
	
	/*
		fn delete_record(
				&self,
				request: DeleteRecordRequest,
			) -> Result<DeleteRecordResponse, SubfieldError> {
			todo!()
		}

		fn put_record(
				&self,
				request: PutRecordRequest,
			) -> Result<PutRecordResponse, SubfieldError> {
			todo!()
		}

		fn subscribe(
				&self,
				request: SubscribeRequest,
			) -> Result<SubscribeResponse, SubfieldError> {
			todo!()
		}

	t	fn unsubscribe(
				&self,
				request: UnsubscribeRequest,
			) -> Result<UnsubscribeResponse, SubfieldError> {
			todo!()
		}
		*/
}
