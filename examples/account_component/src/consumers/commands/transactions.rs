use crate::{handlers::commands::transactions::*, TransactionCategory};
use aqueous::Consumer;

pub struct TransactionsConsumer;

impl TransactionsConsumer {
    pub fn build() -> Consumer {
        let TransactionCategory(category) = crate::TransactionCategory::new();

        Consumer::new(category)
            .extend_handlers((handle_deposit, handle_withdraw))
            .unwrap()
    }
}
