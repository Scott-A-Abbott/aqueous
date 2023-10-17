use aqueous::Message;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub struct Open {
    pub account_id: Uuid,
    pub customer_id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
}
