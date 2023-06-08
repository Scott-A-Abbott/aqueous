use crate::messages::commands::Withdraw;
use aqueous::Message;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WithdrawalRejected {
    pub withdrawal_id: Uuid,
    pub account_id: Uuid,
    pub amount: i64,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub processed_time: OffsetDateTime,
    pub sequence: i64,
}

impl Message for WithdrawalRejected {
    const TYPE_NAME: &'static str = "WithdrawalRejected";
}

impl From<Withdraw> for WithdrawalRejected {
    fn from(withdraw: Withdraw) -> Self {
        Self {
            withdrawal_id: withdraw.withdrawal_id,
            account_id: withdraw.account_id,
            amount: withdraw.amount,
            time: withdraw.time,
            processed_time: OffsetDateTime::now_utc(),
            sequence: 0,
        }
    }
}
