pub use subfield_proto::*;

pub enum SubfieldRequest {
	// system
	Ping(PingRequest),
	
	// records
	GetRecord(GetRecordRequest),
	PutRecord(PutRecordRequest),
	DeleteRecord(DeleteRecordRequest),
	
	// pubsub
	Subscribe(SubscribeRequest),
	Unsubscribe(UnsubscribeRequest),
}

pub enum SubfieldResponse {
	// system
	Ping(PingResponse),
	
	// records
	GetRecord(GetRecordResponse),
	PutRecord(PutRecordResponse),
	DeleteRecord(DeleteRecordResponse),
	
	// pubsub
	Subscribe(SubscribeResponse),
	Unsubscribe(UnsubscribeResponse),
}