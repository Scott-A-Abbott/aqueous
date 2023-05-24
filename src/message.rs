// Should have a derive macro that can generate the message_type needed for MessageData
pub trait Message: Sized {
    fn type_name() -> String;
}

pub struct Metadata;

pub struct Msg<T: Message> {
    pub data: T,
}
impl<T> crate::consumer::FromConsumerState for Msg<T>
where
    for<'de> T: Message + serde::Deserialize<'de>,
{
    type Error = String;
    fn from_consumer_state(message_data: MessageData) -> Result<Self, Self::Error> {
        if &message_data.type_name == &T::type_name() {
            let message =
                serde_json::from_str(&message_data.data).map_err(|err| err.to_string())?;
            Ok(Msg { data: message })
        } else {
            Err(format!(
                "Message Data is not an instance of {}",
                T::type_name()
            ))
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
    pub time: chrono::prelude::NaiveDateTime,
}
