use chrono::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{error::Error, ops::Deref};

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

// Should have a derive macro that can generate the message_type needed for MessageData
pub trait Message: Sized {
    const TYPE_NAME: &'static str;
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Metadata(Map<String, Value>);
impl Metadata {
    const CAUSATION_MESSAGE_STREAM_NAME_KEY: &'static str = "causation_message_stream_name";
    const CAUSATION_MESSAGE_POSITION_KEY: &'static str = "causation_message_position";
    const CAUSATION_MESSAGE_GLOBAL_POSITION_KEY: &'static str = "causation_message_global_position";
    const CORRELATION_STREAM_NAME_KEY: &'static str = "correlation_stream_name";
    const POSITION_KEY: &'static str = "position";
    const GLOBAL_POSITION_KEY: &'static str = "global_position";
    const STREAM_NAME_KEY: &'static str = "stream_name";
    const REPLAY_STREAM_NAME_KEY: &'static str = "correlation_stream_name";
    const TIME_KEY: &'static str = "time";

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

    pub fn stream_name(&self) -> Option<String> {
        let value = self.0.get(Self::STREAM_NAME_KEY)?;
        serde_json::from_value(value.clone()).ok()
    }

    pub fn time(&self) -> Option<NaiveDateTime> {
        let value = self.0.get(Self::TIME_KEY)?;
        let time_string: String = serde_json::from_value(value.clone()).ok()?;

        use std::str::FromStr;
        NaiveDateTime::from_str(&time_string).ok()
    }

    pub fn replay_stream_name(&self) -> Option<String> {
        let value = self.0.get(Self::REPLAY_STREAM_NAME_KEY)?;
        serde_json::from_value(value.clone()).ok()
    }

    pub fn causation_message_stream_name(&self) -> Option<String> {
        let value = self.0.get(Self::CAUSATION_MESSAGE_STREAM_NAME_KEY)?.clone();
        serde_json::from_value(value).ok()
    }

    pub fn causation_message_position(&self) -> Option<i64> {
        let value = self.0.get(Self::CAUSATION_MESSAGE_POSITION_KEY)?.clone();
        serde_json::from_value(value).ok()
    }

    pub fn causation_message_global_position(&self) -> Option<i64> {
        let value = self
            .0
            .get(Self::CAUSATION_MESSAGE_GLOBAL_POSITION_KEY)?
            .clone();
        serde_json::from_value(value).ok()
    }

    pub fn correlation_stream_name(&self) -> Option<String> {
        let value = self.0.get(Self::CORRELATION_STREAM_NAME_KEY)?.clone();
        serde_json::from_value(value).ok()
    }

    pub fn set_causation_message_stream_name(mut self, stream_name: &str) -> Self {
        let key = String::from(Self::CAUSATION_MESSAGE_STREAM_NAME_KEY);
        self.0.insert(key, stream_name.into());
        self
    }

    pub fn set_causation_message_position(mut self, position: i64) -> Self {
        let key = String::from(Self::CAUSATION_MESSAGE_POSITION_KEY);
        self.0.insert(key, position.into());
        self
    }

    pub fn set_causation_message_global_position(mut self, global_position: i64) -> Self {
        let key = String::from(Self::CAUSATION_MESSAGE_GLOBAL_POSITION_KEY);
        self.0.insert(key, global_position.into());
        self
    }

    pub fn set_correlation_stream_name(mut self, stream_name: &str) -> Self {
        let key = String::from(Self::CORRELATION_STREAM_NAME_KEY);
        self.0.insert(key, stream_name.into());
        self
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

    pub fn set_stream_name(mut self, stream_name: &str) -> Self {
        let key = String::from(Self::STREAM_NAME_KEY);
        self.0.insert(key, stream_name.into());
        self
    }

    pub fn set_replay_stream_name(mut self, stream_name: &str) -> Self {
        let key = String::from(Self::REPLAY_STREAM_NAME_KEY);
        self.0.insert(key, stream_name.into());
        self
    }

    pub fn set_time(mut self, time: NaiveDateTime) -> Self {
        let key = String::from(Self::TIME_KEY);
        self.0.insert(key, time.to_string().into());
        self
    }
}

#[derive(Debug, Serialize)]
pub struct Msg<T> {
    pub data: T,
    pub metadata: Option<Metadata>,
}

impl<T> Msg<T> 
where
    for<'de> T: Message + Deserialize<'de>,
{
    pub fn from_data(message_data: MessageData) -> Result<Self, Box<dyn Error>> {
        if &message_data.type_name == T::TYPE_NAME {
            let data = serde_json::from_str(&message_data.data)?;
            let maybe_metadata: Option<Metadata> = serde_json::from_str(&message_data.metadata)?;

            let metadata = maybe_metadata
                .unwrap_or_default()
                .set_position(message_data.position)
                .set_global_position(message_data.global_position)
                .set_time(message_data.time)
                .set_stream_name(&message_data.stream_name);

            Ok(Msg {
                data,
                metadata: Some(metadata),
            })
        } else {
            Err(format!("Message Data is not an instance of {}", T::TYPE_NAME).into())
        }
    }
}

impl<T: Message> Message for Msg<T> {
    const TYPE_NAME: &'static str = T::TYPE_NAME;
}

impl<T> From<T> for Msg<T> {
    fn from(data: T) -> Self {
        Msg {
            data,
            metadata: None,
        }
    }
}

impl<T> Deref for Msg<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
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

impl<T> crate::HandlerParam for Msg<T>
where
    for<'de> T: Message + serde::Deserialize<'de>,
{
    type Error = Box<dyn Error>;

    fn build(message_data: MessageData, _: &crate::HandlerRetainers) -> Result<Self, Self::Error> {
        Msg::from_data(message_data)
    }
}
