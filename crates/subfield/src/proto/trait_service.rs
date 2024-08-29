use crate::*;


	

// the service trait is the user-facing trait for interacting with the protocol
	
#[async_trait]
pub trait SubfieldServiceTrait {
	/**
	 * System
		*/

	async fn echo(&self, subkey: RoutingSubkey, message: &str) -> EchoResponse;

	/*
	Records
	   */

	   /*
	async fn get_record(&self, subkey: Subkey) -> GetRecordResponse;
		*/
	async fn put_record(
		&self,
		subkey: CompleteSubkey,
		record: Record,
	) -> PutRecordResponse;

	async fn put_record_with_keypair(
		&self,
		subkey: CompleteSubkey,
		record: Record,
		keypair: Keypair,
	) -> PutRecordResponse;
	 
	
	/*
	fn delete_record(
		&self,
		request: proto::DeleteRecordRequest,
	) -> SubfieldServiceResult<proto::DeleteRecordResponse>;
	*/

	/*
	Pubsub
	  */
	/*
	fn subscribe(
		&self,
		request: proto::SubscribeRequest,
	) -> SubfieldServiceResult<proto::SubscribeResponse>;

	fn unsubscribe(
		&self,
		request: proto::UnsubscribeRequest,
	) -> SubfieldServiceResult<proto::UnsubscribeResponse>;
	*/
}
