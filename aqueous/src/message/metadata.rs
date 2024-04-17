use crate::stream_name::StreamName;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};
use time::PrimitiveDateTime;

#[derive(Debug, Serialize)]
pub struct Metadata(pub Map<String, Value>);

impl Metadata {
    pub const CAUSATION_MESSAGE_STREAM_NAME_KEY: &'static str = "causation_message_stream_name";
    pub const CAUSATION_MESSAGE_POSITION_KEY: &'static str = "causation_message_position";
    pub const CAUSATION_MESSAGE_GLOBAL_POSITION_KEY: &'static str =
        "causation_message_global_position";
    pub const CORRELATION_STREAM_NAME_KEY: &'static str = "correlation_stream_name";
    pub const POSITION_KEY: &'static str = "position";
    pub const GLOBAL_POSITION_KEY: &'static str = "global_position";
    pub const STREAM_NAME_KEY: &'static str = "stream_name";
    pub const REPLAY_STREAM_NAME_KEY: &'static str = "correlation_stream_name";
    pub const TIME_KEY: &'static str = "time";

    pub fn get_as<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let value = self.0.get(key)?;
        serde_json::from_value(value.clone()).ok()
    }

    pub fn set<T>(mut self, key: &str, value: T) -> Self
    where
        Value: From<T>,
    {
        self.0.insert(key.to_owned(), value.into());
        self
    }

    pub fn follow(metadata: Metadata) -> Self {
        let Metadata(mut map) = metadata;

        map.remove(Self::POSITION_KEY);
        map.remove(Self::GLOBAL_POSITION_KEY);
        map.remove(Self::TIME_KEY);
        map.remove(Self::STREAM_NAME_KEY);

        Self(map)
    }

    pub fn position(&self) -> Option<i64> {
        let value = self.0.get(Self::POSITION_KEY)?;
        serde_json::from_value(value.clone()).ok()
    }

    pub fn set_position(mut self, position: i64) -> Self {
        let key = String::from(Self::POSITION_KEY);
        self.0.insert(key, position.into());
        self
    }

    pub fn global_position(&self) -> Option<i64> {
        let value = self.0.get(Self::GLOBAL_POSITION_KEY)?;
        serde_json::from_value(value.clone()).ok()
    }

    pub fn set_global_position(mut self, global_position: i64) -> Self {
        let key = String::from(Self::GLOBAL_POSITION_KEY);
        self.0.insert(key, global_position.into());
        self
    }

    pub fn stream_name(&self) -> Option<StreamName> {
        let value = self.0.get(Self::STREAM_NAME_KEY)?;
        let stream_name = serde_json::from_value(value.clone()).ok()?;

        Some(StreamName(stream_name))
    }

    pub fn set_stream_name(mut self, stream_name: StreamName) -> Self {
        let key = String::from(Self::STREAM_NAME_KEY);
        self.0.insert(key, stream_name.0.into());
        self
    }

    pub fn time(&self) -> Option<PrimitiveDateTime> {
        let value = self.0.get(Self::TIME_KEY)?;
        serde_json::from_value(value.clone()).ok()
    }

    pub fn set_time(mut self, time: PrimitiveDateTime) -> Self {
        let key = String::from(Self::TIME_KEY);
        let value = serde_json::to_value(time).unwrap();
        self.0.insert(key, value);

        self
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
