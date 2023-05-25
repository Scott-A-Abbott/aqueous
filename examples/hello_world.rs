use aqueous::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let handler = IntoHandler::into_handler(|canceled: Msg<Canceled>| {
        let time = canceled.time;
        println!("Date and time of cancelation: {}", time);

        println!("Debug of message: {:#?}", canceled);
    });
    let mut boxed_handler: Box<dyn Handler> = Box::new(handler);

    const MAX_CONNECTIONS: u32 = 5;
    let message_store =
        MessageStorePg::new("postgres://message_store@localhost/message_store", MAX_CONNECTIONS).await?;

    let messages = message_store
        .get_category_messages("someCategory")
        .await?
        .execute()
        .await?;

    for message_data in messages.into_iter() {
        if boxed_handler.handles_message(&message_data.type_name) {
            boxed_handler.call(message_data);
        }
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
