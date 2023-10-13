use crate::StreamName;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::Row;
use std::{
    error::Error,
    ops::{Deref, DerefMut},
};
use time::PrimitiveDateTime;

#[derive(sqlx::FromRow, Serialize, Debug, Clone)]
pub struct MessageData {
    #[sqlx(rename = "type")]
    pub type_name: String,
    #[sqlx(flatten)]
    pub metadata: Metadata,
    pub data: Value,
}

// Should have a derive macro that can generate the message_type needed for MessageData
pub trait Message: Sized {
    const TYPE_NAME: &'static str;
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Metadata(pub Map<String, Value>);

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Metadata {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        let stream_name_string = row.try_get("stream_name")?;
        let stream_name = StreamName(stream_name_string);
        let global_position = row.try_get("global_position")?;
        let position = row.try_get("position")?;
        let time = row.try_get("time")?;

        let metadata_value: Value = row.try_get("metadata")?;
        let maybe_metadata: Option<Metadata> =
            serde_json::from_value(metadata_value).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let metadata = maybe_metadata
            .unwrap_or_default()
            .set_position(position)
            .set_global_position(global_position)
            .set_stream_name(stream_name)
            .set_time(time);

        Ok(metadata)
    }
}

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

    pub fn position(&self) -> Option<i64> {
        let value = self.0.get(Self::POSITION_KEY)?;
        serde_json::from_value(value.clone()).ok()
    }

    pub fn global_position(&self) -> Option<i64> {
        let value = self.0.get(Self::GLOBAL_POSITION_KEY)?;
        serde_json::from_value(value.clone()).ok()
    }

    pub fn stream_name(&self) -> Option<StreamName> {
        let value = self.0.get(Self::STREAM_NAME_KEY)?;
        let stream_name = serde_json::from_value(value.clone()).ok()?;

        Some(StreamName(stream_name))
    }

    pub fn time(&self) -> Option<PrimitiveDateTime> {
        let value = self.0.get(Self::TIME_KEY)?;
        serde_json::from_value(value.clone()).ok()
    }

    pub fn set_position(mut self, position: i64) -> Self {
        let key = String::from(Self::POSITION_KEY);
        self.0.insert(key, position.into());
        self
    }

    pub fn set_global_position(mut self, global_position: i64) -> Self {
        let key = String::from(Self::GLOBAL_POSITION_KEY);
        self.0.insert(key, global_position.into());
        self
    }

    pub fn set_stream_name(mut self, stream_name: StreamName) -> Self {
        let key = String::from(Self::STREAM_NAME_KEY);
        self.0.insert(key, stream_name.0.into());
        self
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

    pub fn follow(metadata: Metadata) -> Self {
        let Metadata(mut map) = metadata;

        map.remove(Self::POSITION_KEY);
        map.remove(Self::GLOBAL_POSITION_KEY);
        map.remove(Self::TIME_KEY);
        map.remove(Self::STREAM_NAME_KEY);

        Self(map)
    }
}

#[derive(Debug, Serialize)]
pub struct Msg<T> {
    pub data: T,
    pub metadata: Metadata,
}

impl<T> Msg<T>
where
    T: Message,
{
    pub fn from_data(message_data: MessageData) -> Result<Self, Box<dyn Error>>
    where
        for<'de> T: Deserialize<'de>,
    {
        let data = serde_json::from_value(message_data.data)?;
        let metadata = message_data.metadata;

        Ok(Msg { data, metadata })
    }

    pub fn follow<M>(message: Msg<M>) -> Self
    where
        T: From<M>,
    {
        let data = T::from(message.data);
        let metadata = Metadata::follow(message.metadata);

        Self { data, metadata }
    }

    pub fn message_type(&self) -> &str {
        T::TYPE_NAME
    }
}

impl<T> From<T> for Msg<T>
where
    T: Message,
{
    fn from(data: T) -> Self {
        Msg {
            data,
            metadata: Default::default(),
        }
    }
}

impl<T> Deref for Msg<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Msg<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: Clone> Clone for Msg<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            metadata: self.metadata.clone(),
        }
    }
}
