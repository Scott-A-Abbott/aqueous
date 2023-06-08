use aqueous::Message;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::messages::commands::Open;

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

impl From<Open> for Opened {
    fn from(open: Open) -> Self {
        Self {
            account_id: open.account_id,
            customer_id: open.customer_id,
            time: OffsetDateTime::now_utc(),
            processed_time: OffsetDateTime::now_utc(),
        }
    }
}
