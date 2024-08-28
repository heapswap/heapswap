pub use crate::proto::{
	subfield_request::RequestType, subfield_response::ResponseType,
};
use crate::*;

/**
 * wrap request
*/
pub fn proto_wrap_request(request: RequestType) -> SubfieldRequest {
	SubfieldRequest::new(request)
}

/**
 * wrap response
*/

pub fn proto_wrap_response<T>(
	response: T,
) -> Result<SubfieldResponse, SubfieldServiceError>
where
	T: proto::Message + Default + ToResponseType,
{
	let response_type = response.to_response_type()?;
	Ok(SubfieldResponse::new(response_type))
}

pub trait ToResponseType {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError>;
}

// echo

impl ToResponseType for proto::EchoResponse {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Echo(self))
	}
}

impl ToResponseType for proto::echo_response::Success {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Echo(proto::EchoResponse {
			response: Some(proto::echo_response::Response::Success(self)),
		}))
	}
}

impl ToResponseType for proto::echo_response::Failure {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Echo(proto::EchoResponse {
			response: Some(proto::echo_response::Response::Failure(
				self.into(),
			)),
		}))
	}
}

// ping

impl ToResponseType for proto::PingResponse {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Ping(self))
	}
}

impl ToResponseType for proto::ping_response::Success {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Ping(proto::PingResponse {
			response: Some(proto::ping_response::Response::Success(self)),
		}))
	}
}

impl ToResponseType for proto::ping_response::Failure {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Ping(proto::PingResponse {
			response: Some(proto::ping_response::Response::Failure(
				self.into(),
			)),
		}))
	}
}

// get record

impl ToResponseType for proto::GetRecordResponse {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::GetRecord(self))
	}
}

impl ToResponseType for proto::get_record_response::Success {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::GetRecord(proto::GetRecordResponse {
			response: Some(proto::get_record_response::Response::Success(self)),
		}))
	}
}

impl ToResponseType for proto::get_record_response::Failure {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::GetRecord(proto::GetRecordResponse {
			response: Some(proto::get_record_response::Response::Failure(
				self.into(),
			)),
		}))
	}
}

// put record
impl ToResponseType for proto::PutRecordResponse {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::PutRecord(self))
	}
}

impl ToResponseType for proto::put_record_response::Success {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::PutRecord(proto::PutRecordResponse {
			response: Some(proto::put_record_response::Response::Success(self)),
		}))
	}
}

impl ToResponseType for proto::put_record_response::Failure {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::PutRecord(proto::PutRecordResponse {
			response: Some(proto::put_record_response::Response::Failure(
				self.into(),
			)),
		}))
	}
}

// delete record

impl ToResponseType for proto::DeleteRecordResponse {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::DeleteRecord(self))
	}
}

impl ToResponseType for proto::delete_record_response::Success {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::DeleteRecord(proto::DeleteRecordResponse {
			response: Some(proto::delete_record_response::Response::Success(
				self,
			)),
		}))
	}
}

impl ToResponseType for proto::delete_record_response::Failure {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::DeleteRecord(proto::DeleteRecordResponse {
			response: Some(proto::delete_record_response::Response::Failure(
				self.into(),
			)),
		}))
	}
}

// subscribe

impl ToResponseType for proto::SubscribeResponse {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Subscribe(self))
	}
}

impl ToResponseType for proto::subscribe_response::Success {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Subscribe(proto::SubscribeResponse {
			response: Some(proto::subscribe_response::Response::Success(self)),
		}))
	}
}

impl ToResponseType for proto::subscribe_response::Failure {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Subscribe(proto::SubscribeResponse {
			response: Some(proto::subscribe_response::Response::Failure(
				self.into(),
			)),
		}))
	}
}

// unsubscribe

impl ToResponseType for proto::UnsubscribeResponse {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Unsubscribe(self))
	}
}

impl ToResponseType for proto::unsubscribe_response::Success {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Unsubscribe(proto::UnsubscribeResponse {
			response: Some(proto::unsubscribe_response::Response::Success(
				self,
			)),
		}))
	}
}

impl ToResponseType for proto::unsubscribe_response::Failure {
	fn to_response_type(self) -> Result<ResponseType, SubfieldServiceError> {
		Ok(ResponseType::Unsubscribe(proto::UnsubscribeResponse {
			response: Some(proto::unsubscribe_response::Response::Failure(
				self.into(),
			)),
		}))
	}
}
