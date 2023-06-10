use crate::{messages::events::*, Account, AccountCategory};
use aqueous::{EntityStore, HandlerParam, Msg, PgPool};

pub struct Store(pub EntityStore<Account>);

impl HandlerParam for Store {
    fn build(pool: PgPool, _: ()) -> Self {
        let AccountCategory(category) = AccountCategory::new();
        let mut store = EntityStore::build(pool, category);

        store
            .add_projection(apply_opened)
            .add_projection(apply_deposited)
            .add_projection(apply_withdrawn)
            .add_projection(apply_withdrawal_rejected)
            .add_projection(apply_closed);

        Self(store)
    }
}

fn apply_opened(account: &mut Account, opened: Msg<Opened>) {
    account.id = opened.account_id;
    account.customer_id = opened.customer_id;
    account.opened_time = Some(opened.time);
}

fn apply_deposited(account: &mut Account, deposited: Msg<Deposited>) {
    account.id = deposited.account_id;

    let amount = deposited.amount;
    account.deposit(amount);

    account.sequence = deposited.sequence;
}

fn apply_withdrawn(account: &mut Account, withdrawn: Msg<Withdrawn>) {
    account.id = withdrawn.account_id;

    let amount = withdrawn.amount;
    account.withdraw(amount);

    account.sequence = withdrawn.sequence;
}

fn apply_withdrawal_rejected(account: &mut Account, withdrawal_rejected: Msg<WithdrawalRejected>) {
    account.id = withdrawal_rejected.account_id;
    account.sequence = withdrawal_rejected.sequence;
}

fn apply_closed(account: &mut Account, closed: Msg<Closed>) {
    account.id = closed.account_id;
    account.closed_time = Some(closed.time);
}
