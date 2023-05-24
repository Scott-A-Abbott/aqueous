use aqueous::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
pub async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://message_store@localhost/message_store")
        .await
        .unwrap();

    let message_data_rows: Vec<MessageData> =
        sqlx::query_as("SELECT * from get_stream_messages($1);")
            .bind("someCategory-767276cf-3f15-46c4-a8ee-4cd1294f19b9")
            .fetch_all(&pool)
            .await
            .unwrap();

    for message_data in message_data_rows.into_iter() {
        message_handler.call(message_data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Canceled {
    time: DateTime<Utc>,
    user_id: Uuid,
    client_id: Uuid,
}
impl Message for Canceled {
    fn type_name() -> String {
        String::from("Canceled")
    }
}

fn message_handler(canceled: Msg<Canceled>) {
    let time = canceled.data.time;
    println!("Date and time of cancelation: {}", time);
}
