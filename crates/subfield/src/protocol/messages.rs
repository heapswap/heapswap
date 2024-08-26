use crate::*;

pub enum SubfieldRequest {
	// system
	Ping(proto::PingRequest),
	
	// records
	GetRecord(proto::GetRecordRequest),
	PutRecord(proto::PutRecordRequest),
	DeleteRecord(proto::DeleteRecordRequest),
	
	// pubsub
	Subscribe(proto::SubscribeRequest),
	Unsubscribe(proto::UnsubscribeRequest),
}

pub enum SubfieldResponse {
	// system
	Ping(proto::PingResponse),
	
	// records
	GetRecord(proto::GetRecordResponse),
	PutRecord(proto::PutRecordResponse),
	DeleteRecord(proto::DeleteRecordResponse),
	
	// pubsub
	Subscribe(proto::SubscribeResponse),
	Unsubscribe(proto::UnsubscribeResponse),
}