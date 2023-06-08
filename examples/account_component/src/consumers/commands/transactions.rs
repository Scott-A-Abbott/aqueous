use crate::{handlers::commands::transactions::*, TransactionCategory};
use aqueous::Consumer;
use sqlx::PgPool;

pub struct TransactionsConsumer;

impl TransactionsConsumer {
    pub fn build(pool: PgPool) -> Consumer<PgPool, ()> {
        let TransactionCategory(category) = crate::TransactionCategory::new();

        Consumer::new(pool, category)
    }
}
