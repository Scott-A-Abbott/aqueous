use aqueous::Message;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Deposit {
    pub deposit_id: Uuid,
    pub account_id: Uuid,
    pub amount: i64,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
}

impl Message for Deposit {
    const TYPE_NAME: &'static str = "Deposit";
}
