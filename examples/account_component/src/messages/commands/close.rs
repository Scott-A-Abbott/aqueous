use aqueous::Message;
use time::OffsetDateTime;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Close {
    pub account_id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
}

impl Message for Close {
    const TYPE_NAME: &'static str = "Close";
}
