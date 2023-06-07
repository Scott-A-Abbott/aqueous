use aqueous::Message;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Close {
    pub account_id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
}

impl Message for Close {
    const TYPE_NAME: &'static str = "Close";
}
