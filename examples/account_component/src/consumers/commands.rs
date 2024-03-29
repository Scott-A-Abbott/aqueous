pub mod transactions;

pub use transactions::TransactionsConsumer;

use crate::{handlers::commands::*, AccountCommandCategory};
use aqueous::Consumer;

pub struct CommandsConsumer;

impl CommandsConsumer {
    pub fn build() -> Consumer {
        let AccountCommandCategory(category) = AccountCommandCategory::new();

        Consumer::new(category)
            .extend_handlers((handle_open, handle_deposit, handle_withdraw, handle_close))
            .expect("Extend handlers")
    }
}
