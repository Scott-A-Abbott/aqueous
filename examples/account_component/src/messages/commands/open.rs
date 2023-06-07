use aqueous::Message;
use time::OffsetDateTime;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Open {
    pub account_id: Uuid,
    pub customer_id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
}

impl Message for Open {
    const TYPE_NAME: &'static str = "Open";
}
