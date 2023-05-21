// Should have a derive macro that can generate the message_type needed for MessageData
pub trait Message {}

pub struct Metadata;

pub struct Msg<T: Message> {
    pub data: T,
}

pub struct MessageData {
    pub id: String,
    pub metadata: Metadata,
    pub message_type: String,
    pub data: String,
}
