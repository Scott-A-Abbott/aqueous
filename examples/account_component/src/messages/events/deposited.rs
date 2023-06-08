use crate::messages::commands::Deposit;
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

impl From<Deposit> for Deposited {
    fn from(deposit: Deposit) -> Self {
        Self {
            deposit_id: deposit.deposit_id,
            account_id: deposit.account_id,
            amount: deposit.amount,
            time: deposit.time,
            processed_time: OffsetDateTime::now_utc(),
            sequence: 0,
        }
    }
}
