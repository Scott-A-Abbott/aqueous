pub mod transactions;

use crate::{
    messages::{commands::*, events::*},
    AccountCategory, Store, TransactionCategory,
};
use aqueous::{MessageStoreError, Msg, StreamID, WriteMessages};

pub async fn handle_open(
    open: Msg<Open>,
    Store(mut store): Store,
    mut writer: WriteMessages,
    AccountCategory(category): AccountCategory,
) {
    let stream_id = StreamID::new(open.account_id);
    let (account, version) = store.fetch(stream_id.clone()).await;

    if account.was_opened() {
        // ## Add logging
        return;
    }

    let opened: Msg<Opened> = Msg::follow(open);
    let stream_name = category.stream_name(stream_id);

    writer
        .add_message(opened)
        .expected_version(version)
        .execute(stream_name)
        .await
        .unwrap();
}

pub async fn handle_deposit(
    deposit: Msg<Deposit>,
    mut writer: WriteMessages,
    TransactionCategory(category): TransactionCategory,
) {
    let stream_id = StreamID::new(deposit.deposit_id);
    let stream_name = category.stream_name(stream_id);

    let deposit: Msg<Deposit> = Msg::follow(deposit);

    let result = writer
        .add_message(deposit)
        .initial()
        .execute(stream_name)
        .await;

    match result {
        Ok(_) | Err(MessageStoreError::WrongExpectedVersion(_)) => return,
        _ => result.unwrap(),
    };
}

pub async fn handle_withdraw(
    withdraw: Msg<Withdraw>,
    mut writer: WriteMessages,
    TransactionCategory(category): TransactionCategory,
) {
    let stream_id = StreamID::new(withdraw.withdrawal_id);
    let stream_name = category.stream_name(stream_id);

    let withdraw: Msg<Withdraw> = Msg::follow(withdraw);

    let result = writer
        .add_message(withdraw)
        .initial()
        .execute(stream_name)
        .await;

    match result {
        Ok(_) | Err(MessageStoreError::WrongExpectedVersion(_)) => return,
        _ => result.unwrap(),
    };
}

pub async fn handle_close(
    close: Msg<Close>,
    Store(mut store): Store,
    mut writer: WriteMessages,
    AccountCategory(category): AccountCategory,
) {
    let stream_id = StreamID::new(close.account_id);
    let (account, version) = store.fetch(stream_id.clone()).await;

    if account.was_closed() {
        // ## Add logging
        return;
    }

    let closed: Msg<Closed> = Msg::follow(close);
    let stream_name = category.stream_name(stream_id);

    writer
        .add_message(closed)
        .expected_version(version)
        .execute(stream_name)
        .await
        .unwrap();
}
