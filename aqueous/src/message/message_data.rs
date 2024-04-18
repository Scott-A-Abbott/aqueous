use super::{Message, Metadata, Msg};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize)]
pub struct MessageData {
    pub type_name: String,
    pub metadata: Metadata,
    pub data: Value,
}

impl<M: Message + Serialize> TryFrom<Msg<M>> for MessageData {
    type Error = Box<dyn std::error::Error>;

    fn try_from(message: Msg<M>) -> Result<Self, Self::Error> {
        let data = serde_json::to_value(&message.data)?;
        let msg = Self {
            data,
            type_name: M::TYPE_NAME.to_string(),
            metadata: Metadata::follow(message.metadata),
        };

        Ok(msg)
    }
}
