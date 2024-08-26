use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubfieldServiceError {}

pub trait SubfieldServiceTrait {
	/**
	 * System
	*/
	fn ping(&self, request: proto::PingRequest) -> Result<proto::PingResponse, SubfieldServiceError>;
	
	/**
	 * Records
	*/
	fn get_record(&self, request: proto::GetRecordRequest) -> Result<Receiver<proto::GetRecordResponse>, SubfieldServiceError>;
	
	fn put_record(&self, request: proto::PutRecordRequest) -> Result<proto::PutRecordResponse, SubfieldServiceError>;
	
	fn delete_record(&self, request: proto::DeleteRecordRequest) -> Result<proto::DeleteRecordResponse, SubfieldServiceError>;
	
	/**
	 * Pubsub
	*/
	fn subscribe(&self, request: proto::SubscribeRequest) -> Result<Receiver<proto::SubscribeResponse>, SubfieldServiceError>;
	
	fn unsubscribe(&self, request: proto::UnsubscribeRequest) -> Result<proto::UnsubscribeResponse, SubfieldServiceError>;
}