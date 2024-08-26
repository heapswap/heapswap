use crate::*;
use chrono::{TimeZone, Utc, DateTime};
use std::time::Duration;
use thiserror::Error;
// use std::fmt::Display;

#[derive(Debug, Error, strum::Display)]
pub enum TimestampError {
	InvalidTimestamp
}

pub fn timestamp_proto_to_datetime(timestamp: proto::google::protobuf::Timestamp) -> Result<DateTime<Utc>, TimestampError> {
    let seconds = timestamp.seconds;
    let nanos = timestamp.nanos;
    let datetime = Utc.timestamp_opt(seconds, nanos as u32).single().ok_or(TimestampError::InvalidTimestamp).map_err(|e| TimestampError::InvalidTimestamp)?;
    Ok(datetime)
}

pub fn datetime_to_timestamp_proto(datetime: DateTime<Utc>) -> proto::google::protobuf::Timestamp {
    proto::google::protobuf::Timestamp {
        seconds: datetime.timestamp(),
        nanos: datetime.timestamp_subsec_nanos() as i32,
    }
}

pub fn now_timestamp_proto() -> proto::google::protobuf::Timestamp {
    datetime_to_timestamp_proto(Utc::now())
}