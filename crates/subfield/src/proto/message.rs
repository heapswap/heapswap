use crate::*;

pub enum SubfieldRequest {
	// System
	Ping(PingRequest),
	Echo(EchoRequest),

	// Record
	GetRecord(GetRecordRequest),
	PutRecord(PutRecordRequest),
	DeleteRecord(DeleteRecordRequest),

	// Pubsub
	Subscribe(SubscribeRequest),
	Unsubscribe(UnsubscribeRequest),
}


pub enum SubfieldResponse  {
	// System
	Ping(PingResponse),
	Echo(EchoResponse),

	// Record
	GetRecord(GetRecordResponse),
	PutRecord(PutRecordResponse),
	DeleteRecord(DeleteRecordResponse),

	// Pubsub
	Subscribe(SubscribeResponse),
	Unsubscribe(UnsubscribeResponse),
}
