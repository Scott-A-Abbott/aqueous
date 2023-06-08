use crate::{
    messages::{commands::*, events::*},
    AccountCategory, Store,
};
use aqueous::{Msg, StreamID, WriteMessages};
use sqlx::PgPool;

pub async fn handle_deposit(
    deposit: Msg<Deposit>,
    Store(mut store): Store<PgPool>,
    AccountCategory(category): AccountCategory,
    mut writer: WriteMessages<PgPool>,
) {
    let stream_id = StreamID::new(deposit.account_id);
    let stream_name = category.stream_name(stream_id);

    let (account, version) = store.fetch(stream_name.clone()).await;

    let sequence = deposit
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.global_position())
        .unwrap();

    if account.has_processed(sequence) {
        // ## Add logging
        return;
    }

    let mut deposited: Msg<Deposited> = Msg::follow(deposit);
    deposited.sequence = sequence;

    writer
        .with_message(deposited)
        .expected_version(version)
        .execute(stream_name)
        .await
        .unwrap();
}
