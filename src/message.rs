use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;

// Should have a derive macro that can generate the message_type needed for MessageData
pub trait Message: Sized {
    fn type_name() -> String;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawMetadata {
    pub causation_message_stream_name: Option<String>,
    pub causation_message_position: Option<i64>,
    pub causation_message_global_position: Option<i64>,
    pub correlation_stream_name: Option<String>,
    pub reply_stream_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub stream_name: String,
    pub position: i64,
    pub global_position: i64,
    pub causation_message_stream_name: Option<String>,
    pub causation_message_position: Option<i64>,
    pub causation_message_global_position: Option<i64>,
    pub correlation_stream_name: Option<String>,
    pub reply_stream_name: Option<String>,
    pub time: NaiveDateTime,
}

impl Metadata {
    pub fn new(message_data: MessageData, raw_metadata: RawMetadata) -> Self {
        Metadata {
            stream_name: message_data.stream_name,
            position: message_data.position,
            global_position: message_data.global_position,
            causation_message_stream_name: raw_metadata.causation_message_stream_name,
            causation_message_position: raw_metadata.causation_message_position,
            causation_message_global_position: raw_metadata.causation_message_global_position,
            correlation_stream_name: raw_metadata.correlation_stream_name,
            reply_stream_name: raw_metadata.reply_stream_name,
            time: message_data.time,
        }
    }
}

#[derive(Debug)]
pub struct Msg<T: Message> {
    pub data: T,
    pub metadata: Metadata,
}
impl<T: Message> std::ops::Deref for Msg<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> crate::HandlerParam for Msg<T>
where
    for<'de> T: Message + serde::Deserialize<'de>,
{
    type Error = Box<dyn Error>;

    fn initialize(message_data: MessageData) -> Result<Self, Self::Error> {
        if &message_data.type_name == &T::type_name() {
            let data = serde_json::from_str(&message_data.data)?;
            let raw_metadata = serde_json::from_str(&message_data.metadata)?;

            let metadata = Metadata::new(message_data, raw_metadata);

            Ok(Msg { data, metadata })
        } else {
            Err(format!("Message Data is not an instance of {}", T::type_name()).into())
        }
    }
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct MessageData {
    pub id: String,
    pub stream_name: String,
    #[sqlx(rename = "type")]
    pub type_name: String,
    pub position: i64,
    pub global_position: i64,
    pub metadata: String,
    pub data: String,
    pub time: NaiveDateTime,
}
