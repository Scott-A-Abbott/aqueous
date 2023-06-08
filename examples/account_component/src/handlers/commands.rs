mod transactions;

use crate::{
    messages::{commands::*, events::*},
    AccountCategory, Store, TransactionCategory,
};
use aqueous::{Msg, StreamID, WriteMessages};
use sqlx::PgPool;

pub async fn handle_open(
    open: Msg<Open>,
    Store(mut store): Store<PgPool>,
    mut writer: WriteMessages<PgPool>,
    AccountCategory(category): AccountCategory,
) {
    let account_stream_id = StreamID::new(open.account_id);
    let stream_name = category.stream_name(account_stream_id);

    let (account, version) = store.fetch(stream_name.clone()).await;

    if account.was_opened() {
        // ## Add logging
        return;
    }

    let opened: Msg<Opened> = Msg::follow(open);

    writer
        .with_message(opened)
        .expected_version(version)
        .execute(stream_name)
        .await
        .unwrap();
}

pub async fn handle_deposit(
    deposit: Msg<Deposit>,
    mut writer: WriteMessages<PgPool>,
    TransactionCategory(category): TransactionCategory,
) {
    let stream_id = StreamID::new(deposit.deposit_id);
    let stream_name = category.stream_name(stream_id);

    let deposit: Msg<Deposit> = Msg::follow(deposit);

    let _ = writer.with_message(deposit).initial().execute(stream_name);
}
