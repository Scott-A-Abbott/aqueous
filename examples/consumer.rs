use aqueous::*;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::error::Error;
use time::OffsetDateTime;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    const CATEGORY: &'static str = "someAccountCategory:command";

    Consumer::new(pool, CATEGORY)
        .identifier("someIdentifier")
        .add_handler(handler)
        .position_update_interval(8)
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
        time: OffsetDateTime::now_utc(),
    };

    let stream_name = format!("someAccountCategory-{}", deposit.account_id);

    let (account, version) = store.fetch(&stream_name).await;

    println!(
        "Depositing {} into account {} with balance {}",
        deposited.amount, account.id, account.balance
    );

    writer
        .with_message(deposited)
        .expected_version(version)
        .execute(&stream_name)
        .await
        .unwrap();
}

#[derive(Debug, Clone, Default)]
struct AccountEntity {
    pub id: Uuid,
    pub balance: i64,
}

struct AccountStore(Store<AccountEntity, PgPool>);

impl HandlerParam<PgPool> for AccountStore {
    fn build(message_data: MessageData, executor: PgPool) -> Self {
        let mut store = Store::build(message_data, executor.clone());

        store.with_projection(|account: &mut AccountEntity, message: Msg<Deposited>| {
            account.id = message.account_id;
            account.balance += message.amount;
        });

        Self(store)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Deposit {
    account_id: Uuid,
    amount: i64,
    time: OffsetDateTime,
}
impl Message for Deposit {
    const TYPE_NAME: &'static str = "Deposit";
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Deposited {
    account_id: Uuid,
    amount: i64,
    time: OffsetDateTime,
}
impl Message for Deposited {
    const TYPE_NAME: &'static str = "Deposited";
}
