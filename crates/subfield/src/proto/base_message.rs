use crate::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubfieldRequest {
	// System
	Ping(PingRequest), // oneshot
	Echo(EchoRequest), // oneshot

	// Record
	GetRecord(GetRecordRequest), // oneshot
	PutRecord(PutRecordRequest), // oneshot
	DeleteRecord(DeleteRecordRequest), // oneshot

	// Pubsub
	Subscribe(SubscribeRequest), // streaming
	Unsubscribe(UnsubscribeRequest), // streaming
}

impl SubfieldRequest {
	pub fn is_streaming(&self) -> bool {
		matches!(self, 
			SubfieldRequest::Subscribe(_) | SubfieldRequest::Unsubscribe(_)
		)
	}
	
	pub fn is_oneshot(&self) -> bool {
		!self.is_streaming()
	}
	
	
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubfieldResponse  {
	// System
	Ping(PingResponse), // oneshot
	Echo(EchoResponse), // oneshot

	// Record
	GetRecord(GetRecordResponse), // oneshot
	PutRecord(PutRecordResponse), // oneshot
	DeleteRecord(DeleteRecordResponse), // oneshot

	// Pubsub
	Subscribe(SubscribeResponse), // streaming
	Unsubscribe(UnsubscribeResponse), // streaming
}


impl SubfieldResponse {
	pub fn is_streaming(&self) -> bool {
		matches!(self, 
			SubfieldResponse::Subscribe(_) | SubfieldResponse::Unsubscribe(_)
		)
	}
	
	pub fn is_oneshot(&self) -> bool {
		!self.is_streaming()
	}
}