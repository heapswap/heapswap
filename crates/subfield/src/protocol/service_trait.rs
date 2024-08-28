use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubfieldServiceError {
	IncompleteSubkey,
	NoConnectedPeers,
	NoLocalPeer,
	SelfIsClosest,
	UnexpectedResponseType,
}

pub type SubfieldServiceResult<T> = Result<T, SubfieldServiceError>;
pub type SubfieldServiceStreamingResult<T> =
	Result<Receiver<T>, SubfieldServiceError>;

#[async_trait]
pub trait SubfieldService {
	/**
	 * System
		*/
	/*
	async fn ping(
		&self,
		request: proto::PingRequest,
	) -> SubfieldServiceResult<proto::PingResponse>;
	*/

	async fn echo(
		&self,
		message: &str,
	) -> SubfieldServiceResult<proto::EchoResponse>;

	/*
	Records
	   */

	async fn get_record(
		&self,
		subkey: protocol::Subkey,
	) -> SubfieldServiceResult<proto::GetRecordResponse>;
	

	
	async fn put_record(
		&self,
		subkey: protocol::Subkey,
		record: proto::Record,
	) -> SubfieldServiceResult<proto::PutRecordResponse>;

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
