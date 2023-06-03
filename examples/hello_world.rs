use aqueous::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::error::Error;
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

    let stream_name = format!("someAccountCategory:command-{}", deposit.account_id);

    WriteMessages::new(pool.clone())
        .with_batch(&batch)
        .execute(&stream_name)
        .await?;

    // HANDLING COMMANDS
    let handler = |deposit: Msg<Deposit>,
                   mut writer: WriteMessages<PgPool>,
                   AccountStore(mut store): AccountStore| async move {
        let deposited = Deposited {
            account_id: deposit.account_id,
            amount: deposit.amount,
            time: Utc::now(),
        };

        let stream_name = format!("someAccountCategory-{}", deposit.account_id);

        let (account, version) = store.fetch(&stream_name).await;

        println!("Account Balance: {}", account.balance);

        writer
            .with_message(deposited)
            .expected_version(version)
            .execute(&stream_name)
            .await
            .unwrap();
    };

    let mut boxed_handler: Box<dyn Handler<PgPool>> = Box::new(handler.into_handler());

    let messages = GetCategoryMessages::new(pool.clone(), "someAccountCategory:command")
        .execute()
        .await?;

    println!("Message count: {}", messages.len());

    for message_data in messages.into_iter() {
        boxed_handler.call(message_data, pool.clone());
    }

    Ok(())
}

#[derive(Default)]
struct AccountEntity {
    pub balance: i64,
}

struct AccountStore(Store<AccountEntity, PgPool>);
impl HandlerParam<PgPool> for AccountStore {
    fn build(message_data: MessageData, executor: PgPool) -> Self {
        let mut store = Store::build(message_data, executor.clone());

        store.with_projection(|account: &mut AccountEntity, message: Msg<Deposited>| {
            account.balance += message.amount;
        });

        Self(store)
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
