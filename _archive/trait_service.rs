use crate::*;

// the service trait is the user-facing trait for interacting with the protocol

#[async_trait]
pub trait SubfieldServiceTrait {
	/*
	System
	*/

	async fn echo(&self, key: CompleteKey, message: &str) -> Result<String, SubfieldError>;

	/*
	Records
	*/

	/*
	async fn get_record(&self, key: CompleteKey) -> Record;

	async fn put_record(
		&self,
		key: CompleteKey,
		record: Record,
	) -> PutRecordResponse;
	*/

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
