use aqueous::Message;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use crate::messages::commands::Close;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Closed {
    pub account_id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub processed_time: OffsetDateTime,
}

impl Message for Closed {
    const TYPE_NAME: &'static str = "Closed";
}

impl From<Close> for Closed {
    fn from(close: Close) -> Self {
        Self {
            account_id: close.account_id,
            time: close.time,
            processed_time: OffsetDateTime::now_utc(),
        }
    }
}
