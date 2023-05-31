use aqueous::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let handler =
        IntoHandler::into_handler(|deposit: Msg<Deposit>, mut some_param: Retain<SomeParam>| {
            some_param.add(5);
            println!("Some param: {:?}", some_param.value());

            println!("Deposit: {:#?}", deposit);
        });
    let mut boxed_handler: Box<dyn Handler> = Box::new(handler);

    const MAX_CONNECTIONS: u32 = 5;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    let messages = GetCategoryMessages::new(pool.clone(), "someAccountCategory")
        .execute()
        .await?;

    println!("Message count: {}", messages.len());

    for message_data in messages.into_iter() {
        if boxed_handler.handles_message(&message_data.type_name) {
            boxed_handler.call(message_data);
        }
    }

    // WRITING A COMMAND
    let deposit = Deposit {
        account_id: Uuid::new_v4(),
        amount: 10,
        time: Utc::now(),
    };

    let stream_name = format!("someAccountCategory-{}", deposit.account_id);

    let last_position = WriteMessages::new(
        pool.clone(),
        &stream_name,
    )
    .with_message(deposit)
    .execute()
    .await?;

    println!("Last position written: {}", last_position);

    // WRITING A BATCH OF EVENTS
    let deposited = Deposited {
        account_id: Uuid::new_v4(),
        amount: 10,
        time: Utc::now(),
    };

    let batch: Vec<Deposited> = (0..10).into_iter().map(|_| deposited.clone()).collect();

    let stream_name = format!("someAccountCategory-{}", deposited.account_id);

    WriteMessages::new(
        pool.clone(),
        &stream_name,
    )
        .with_batch(batch)
        .execute()
        .await?;

    // PROJECTING AN ENTITY
    let mut store = Store::new(pool.clone());
    store.with_projection(|account: &mut AccountEntity, message: Msg<Deposited>| {
        account.balance += message.amount;
    });

    let account = store
        .fetch(&stream_name)
        .await;

    println!("Account balance: {}", account.balance);

    Ok(())
}

#[derive(Default)]
struct AccountEntity {
    pub balance: i64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Deposit {
    account_id: Uuid,
    amount: i64,
    time: DateTime<Utc>,
}
impl Message for Deposit {
    const TYPE_NAME: &'static str = "Deposit";
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Deposited {
    account_id: Uuid,
    amount: i64,
    time: DateTime<Utc>,
}
impl Message for Deposited {
    const TYPE_NAME: &'static str = "Deposited";
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
    fn build(_: MessageData, _: &HandlerRetainers) -> Result<Self, Self::Error> {
        Ok(Self(10))
    }
}
