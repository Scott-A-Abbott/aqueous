use crate::{
    messages::{commands::*, events::*},
    AccountCategory, Store,
};
use aqueous::{Msg, StreamID, WriteMessages};
use tracing::{info, instrument};

#[instrument(skip_all, target = "account_component")]
pub async fn handle_deposit(
    deposit: Msg<Deposit>,
    Store(mut store): Store,
    AccountCategory(category): AccountCategory,
    mut writer: WriteMessages,
) {
    let stream_id = StreamID::new(deposit.account_id);
    let (account, version) = store.fetch(stream_id.clone()).await;

    let sequence = deposit
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.global_position())
        .unwrap();

    if account.has_processed(sequence) {
        info!(
            target: "ignored",
            "Command ignored (Command: {}, Account ID: {}, Account Sequence: {}, Deposit Sequence: {})",
            deposit.message_type(), account.id, account.sequence, sequence
        );
        return;
    }

    let mut deposited: Msg<Deposited> = Msg::follow(deposit);
    deposited.sequence = sequence;

    let stream_name = category.stream_name(stream_id);

    writer
        .add_message(deposited)
        .expected_version(version)
        .execute(stream_name)
        .await
        .unwrap();
}

#[instrument(skip_all, target = "account_component")]
pub async fn handle_withdraw(
    withdraw: Msg<Withdraw>,
    Store(mut store): Store,
    AccountCategory(category): AccountCategory,
    mut writer: WriteMessages,
) {
    let stream_id = StreamID::new(withdraw.account_id);
    let (account, version) = store.fetch(stream_id.clone()).await;

    let sequence = withdraw
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.global_position())
        .unwrap();

    if account.has_processed(sequence) {
        info!(
            target: "ignored",
            "Command ignored (Command: {}, Account ID: {}, Account Sequence: {}, Deposit Sequence: {})",
            withdraw.message_type(), account.id, account.sequence, sequence
        );
        return;
    }

    let stream_name = category.stream_name(stream_id);

    if !account.has_sufficient_funds(withdraw.amount) {
        let mut withdrawal_rejected: Msg<WithdrawalRejected> = Msg::follow(withdraw);
        withdrawal_rejected.sequence = sequence;

        writer
            .add_message(withdrawal_rejected)
            .expected_version(version)
            .execute(stream_name)
            .await
            .unwrap();

        return;
    }

    let mut withdrawn: Msg<Withdrawn> = Msg::follow(withdraw);
    withdrawn.sequence = sequence;

    writer
        .add_message(withdrawn)
        .expected_version(version)
        .execute(stream_name)
        .await
        .unwrap();
}
