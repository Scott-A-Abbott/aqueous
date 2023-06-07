use aqueous::*;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn handler(
    deposit: Msg<Deposit>,
    mut writer: WriteMessages<PgPool>,
    AccountStore(mut store): AccountStore,
    AccountCategory(category): AccountCategory,
) {
    let account_stream_id = StreamID::new(deposit.account_id);
    let stream_name = category.stream_name(account_stream_id);
    let deposited = Msg::<Deposited>::follow(deposit);

    let (account, version) = store.fetch(stream_name.clone()).await;

    println!(
        "Depositing {} into account {} with balance {}",
        deposited.amount, account.id, account.balance
    );

    writer
        .with_message(deposited)
        .expected_version(version)
        .execute(stream_name)
        .await
        .unwrap();
}

#[derive(Debug, Clone, Default)]
pub struct AccountEntity {
    pub id: Uuid,
    pub balance: i64,
}

pub struct AccountStore(EntityStore<AccountEntity, PgPool>);

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

pub struct AccountCategory(Category);

impl HandlerParam<PgPool, ()> for AccountCategory {
    fn build(_: PgPool, _: ()) -> Self {
        let category = Category::new("someAccountCategory");
        AccountCategory(category)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Deposit {
    pub account_id: Uuid,
    pub amount: i64,
    pub time: OffsetDateTime,
}

impl Message for Deposit {
    const TYPE_NAME: &'static str = "Deposit";
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Deposited {
    pub account_id: Uuid,
    pub amount: i64,
    #[serde(with = "time::serde::iso8601")]
    pub time: OffsetDateTime,
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
