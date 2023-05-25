use aqueous::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut handler = IntoHandler::into_handler(|canceled: Msg<Canceled>| {
        let time = canceled.data.time;
        println!("Date and time of cancelation: {}", time);

        println!("Debug of message: {:#?}", canceled);
    });

    let message_store =
        MessageStorePg::new("postgres://message_store@localhost/message_store", 5).await?;
    let messages = message_store
        .get_stream_messages("someCategory-767276cf-3f15-46c4-a8ee-4cd1294f19b9")
        .await?
        .execute()
        .await?;

    for message_data in messages.into_iter() {
        handler.call(message_data);
    }

    Ok(())
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

#[derive(Serialize, Deserialize, Debug)]
struct Deposit {
    amount: i64,
}
impl Message for Deposit {
    fn type_name() -> String {
        String::from("Deposit")
    }
}

fn handle_deposit(deposit: Msg<Deposit>) {
    let amount = deposit.data.amount;
    println!("Amount to deposit: {}", amount);
}
