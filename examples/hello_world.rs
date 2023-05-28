use aqueous::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let handler =
        IntoHandler::into_handler(|canceled: Msg<Canceled>, mut some_param: Retain<SomeParam>| {
            let time = canceled.time;
            some_param.add(5);

            println!("Some param: {:?}", some_param.value());
            println!("Date and time of cancelation: {}", time);
        });
    let mut boxed_handler: Box<dyn Handler> = Box::new(handler);

    let handler2 = IntoHandler::into_handler(handle_message::<Canceled>);
    let mut boxed_handler2: Box<dyn Handler> = Box::new(handler2);

    const MAX_CONNECTIONS: u32 = 5;
    let message_store = MessageStorePg::new(
        "postgres://message_store@localhost/message_store",
        MAX_CONNECTIONS,
    )
    .await?;

    let messages = message_store
        .get_category_messages("someCategory")
        .await?
        .execute()
        .await?;

    for message_data in messages.into_iter() {
        if boxed_handler.handles_message(&message_data.type_name) {
            boxed_handler.call(message_data.clone());
            boxed_handler.call(message_data.clone());

            println!("");

            boxed_handler2.call(message_data);
        }
    }

    let deposit_json = serde_json::json!({
        "amount": 10,
        "time": chrono::prelude::Utc::now().to_string(),
    });
    let payload = MessagePayload {
        data: deposit_json.to_string(),
        message_type: String::from("Deposit"),
        metadata: None,
    };

    let last_position = message_store
        .write_messages("someAccountCategory-745D49F3-CB89-4EE9-958D-1BA63E35A061")
        .await?
        .with_message(payload)
        .execute()
        .await?;

    println!("Last position written: {}", last_position);

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
    const TYPE_NAME: &'static str = "Canceled";
}

#[derive(Debug)]
struct SomeParam(i32);
impl SomeParam {
    fn add(&mut self, x: i32) {
        self.0 += x;
    }
    fn value(&self) -> i32 {
        self.0
    }
}
impl HandlerParam for SomeParam {
    type Error = Box<dyn Error>;
    fn build(_: MessageData, _: &mut HandlerRetainers) -> Result<Self, Self::Error> {
        Ok(Self(10))
    }
}

fn handle_message<M: Message + std::fmt::Debug>(message: Msg<M>) {
    println!("Message: {:#?}", message)
}
