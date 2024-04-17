use super::{Message, MessageData, Metadata};
use serde::{Deserialize, Serialize};
use std::error::Error;

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
