pub mod transactions;

pub use transactions::TransactionsConsumer;

use crate::{handlers::commands::*, AccountCommandCategory};
use aqueous::Consumer;
use sqlx::PgPool;

pub struct CommandsConsumer;

impl CommandsConsumer {
    pub fn build(pool: PgPool) -> Consumer<PgPool, ()> {
        let AccountCommandCategory(category) = crate::AccountCommandCategory::new();

        Consumer::new(pool, category).add_handlers((
            handle_open,
            handle_deposit,
            handle_withdraw,
            handle_close,
        ))
    }
}
