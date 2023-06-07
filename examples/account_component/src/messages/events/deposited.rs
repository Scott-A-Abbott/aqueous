use aqueous::Message;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Deposited {
    pub deposit_id: Uuid,
    pub account_id: Uuid,
    pub amount: i64,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub processed_time: OffsetDateTime,
    pub sequence: i64,
}

impl Message for Deposited {
    const TYPE_NAME: &'static str = "Deposited";
}
