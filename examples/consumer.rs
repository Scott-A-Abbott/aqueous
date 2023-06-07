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

    let category = Category::new_command("someAccountCategory");

    Consumer::new(pool, category)
        .identifier(CategoryType::new("someIdentifier"))
        .add_handler(handler)
        .start()
        .await;

    Ok(())
}

async fn handler(
    deposit: Msg<Deposit>,
    mut writer: WriteMessages<PgPool>,
    AccountStore(mut store): AccountStore,
    AccountCategory(category): AccountCategory,
) {
    let account_stream_id = StreamID::new(deposit.account_id);
    let stream_name = category.stream_name(account_stream_id);

    let (account, version) = store.fetch(stream_name.clone()).await;

    println!(
        "Depositing {} into account {} with balance {}",
        deposit.amount, account.id, account.balance
    );

    let deposited = Msg::<Deposited>::follow(deposit);

    writer
        .with_message(deposited)
        .expected_version(version)
        .execute(stream_name)
        .await
        .unwrap();
}

#[derive(Debug, Clone, Default)]
struct AccountEntity {
    pub id: Uuid,
    pub balance: i64,
}

struct AccountStore(EntityStore<AccountEntity, PgPool>);

impl HandlerParam<PgPool, ()> for AccountStore {
    fn build(executor: PgPool, settings: ()) -> Self {
        let mut store = EntityStore::build(executor.clone(), settings);

        store.with_projection(|account: &mut AccountEntity, message: Msg<Deposited>| {
            account.id = message.account_id;
            account.balance += message.amount;
        });

        Self(store)
    }
}

struct AccountCategory(Category);

impl HandlerParam<PgPool, ()> for AccountCategory {
    fn build(_: PgPool, _: ()) -> Self {
        let category = Category::new("someAccountCategory");
        AccountCategory(category)
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

impl From<Deposit> for Deposited {
    fn from(deposit: Deposit) -> Self {
        let account_id = deposit.account_id;
        let amount = deposit.amount;
        let time = OffsetDateTime::now_utc();

        Self {
            account_id,
            amount,
            time,
        }
    }
}
