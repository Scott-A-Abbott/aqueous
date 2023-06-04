use aqueous::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    const MAX_CONNECTIONS: u32 = 1;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    const CATEGORY: &'static str = "someAccountCategory:command";

    Consumer::new(pool, CATEGORY)
        .add_handler(handler)
        .start()
        .await;

    Ok(())
}

async fn handler(
    deposit: Msg<Deposit>,
    mut writer: WriteMessages<PgPool>,
    AccountStore(mut store): AccountStore,
) {
    let deposited = Deposited {
        account_id: deposit.account_id,
        amount: deposit.amount,
        time: Utc::now(),
    };

    let stream_name = format!("someAccountCategory-{}", deposit.account_id);

    let (account, version) = store.fetch(&stream_name).await;

    writer
        .with_message(deposited)
        .expected_version(version)
        .execute(&stream_name)
        .await
        .unwrap();
}

#[derive(Debug, Clone, Default)]
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