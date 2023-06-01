use aqueous::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, ops::DerefMut};
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    const MAX_CONNECTIONS: u32 = 5;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    // WRITING A COMMANDS
    let account_id = Uuid::new_v4();
    let deposit = Deposit {
        account_id,
        amount: 10,
        time: Utc::now(),
    };

    let batch = (0..10)
        .into_iter()
        .map(|_| deposit.clone())
        .collect::<Vec<_>>();

    let stream_name = format!("someAccountCategory-{}", deposit.account_id);

    WriteMessages::new(pool.clone())
        .with_batch(&batch)
        .execute(&stream_name)
        .await?;

    // HANDLING COMMANDS
    let handler = IntoHandler::into_handler(
        |deposit: Msg<Deposit>,
         AccountStore(mut store): AccountStore,
         mut some_param: Res<SomeParam>,
         mut writer: WriteMessages<sqlx::PgPool>| {
            let SomeParam(value) = some_param.deref_mut();
            *value += 5;
            println!("Some param: {}", value);

            let deposited = Deposited {
                account_id: deposit.account_id,
                amount: deposit.amount,
                time: Utc::now(),
            };

            let stream_name = format!("someAccountCategory-{}", deposit.account_id);

            tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    let (account, version) = store.fetch(&stream_name).await;

                    println!("Account Balance: {}", account.balance);

                    writer
                        .with_message(deposited)
                        .expected_version(version)
                        .execute(&stream_name)
                        .await
                        .unwrap();
                });
            });
        },
    );
    handler.resources.insert(pool.clone());

    let mut boxed_handler: Box<dyn Handler> = Box::new(handler);

    let messages = GetCategoryMessages::new(pool.clone(), "someAccountCategory")
        .execute()
        .await?;

    println!("Message count: {}", messages.len());

    for message_data in messages.into_iter() {
        if boxed_handler.handles_message(&message_data.type_name) {
            boxed_handler.call(message_data);
        }
    }

    Ok(())
}

#[derive(Default)]
struct AccountEntity {
    pub balance: i64,
}

struct AccountStore(Store<AccountEntity, sqlx::PgPool>);
impl HandlerParam for AccountStore {
    type Error = Box<dyn Error>;

    fn build(message_data: MessageData, resources: &HandlerResources) -> Result<Self, Self::Error> {
        let mut store = Store::build(message_data, resources)?;

        store.with_projection(|account: &mut AccountEntity, message: Msg<Deposited>| {
            account.balance += message.amount;
        });

        Ok(Self(store))
    }
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

impl HandlerParam for SomeParam {
    type Error = Box<dyn Error>;
    fn build(_: MessageData, _: &HandlerResources) -> Result<Self, Self::Error> {
        Ok(Self(10))
    }
}
