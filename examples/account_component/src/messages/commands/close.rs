use aqueous::Message;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub struct Close {
    pub account_id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
}
