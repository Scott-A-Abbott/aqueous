use aqueous::Message;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Opened {
    pub account_id: Uuid,
    pub customer_id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub processed_time: OffsetDateTime,
}

impl Message for Opened {
    const TYPE_NAME: &'static str = "Opened";
}
