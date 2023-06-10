use crate::{
    messages::{commands::*, events::*},
    AccountCategory, Store,
};
use aqueous::{Msg, StreamID, WriteMessages};

pub async fn handle_deposit(
    deposit: Msg<Deposit>,
    Store(mut store): Store,
    AccountCategory(category): AccountCategory,
    mut writer: WriteMessages,
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

pub async fn handle_withdraw(
    withdraw: Msg<Withdraw>,
    Store(mut store): Store,
    AccountCategory(category): AccountCategory,
    mut writer: WriteMessages,
) {
    let stream_id = StreamID::new(withdraw.account_id);
    let stream_name = category.stream_name(stream_id);

    let (account, version) = store.fetch(stream_name.clone()).await;

    let sequence = withdraw
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.global_position())
        .unwrap();

    if account.has_processed(sequence) {
        // ## Add logging
        return;
    }

    if !account.has_sufficient_funds(withdraw.amount) {
        let mut withdrawal_rejected: Msg<WithdrawalRejected> = Msg::follow(withdraw);
        withdrawal_rejected.sequence = sequence;

        writer
            .with_message(withdrawal_rejected)
            .expected_version(version)
            .execute(stream_name)
            .await
            .unwrap();

        return;
    }

    let mut withdrawn: Msg<Withdrawn> = Msg::follow(withdraw);
    withdrawn.sequence = sequence;

    writer
        .with_message(withdrawn)
        .expected_version(version)
        .execute(stream_name)
        .await
        .unwrap();
}
