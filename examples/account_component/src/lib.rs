pub mod account;
pub mod consumers;
pub mod handlers;
pub mod messages;
pub mod store;

pub use account::Account;
pub use store::Store;

use aqueous::{Category, Connection, HandlerParam};

pub struct AccountCategory(pub Category);

impl AccountCategory {
    pub fn new() -> Self {
        let category = Category::new("account");
        Self(category)
    }
}

impl HandlerParam for AccountCategory {
    fn build(_: Connection, _: ()) -> Self {
        Self::new()
    }
}

pub struct AccountCommandCategory(pub Category);

impl AccountCommandCategory {
    pub fn new() -> Self {
        let category = Category::new_command("account");
        Self(category)
    }
}

impl HandlerParam for AccountCommandCategory {
    fn build(_: Connection, _: ()) -> Self {
        Self::new()
    }
}

pub struct TransactionCategory(pub Category);

impl TransactionCategory {
    pub fn new() -> Self {
        let category = Category::new("accountTransaction");
        Self(category)
    }
}

impl HandlerParam for TransactionCategory {
    fn build(_: Connection, _: ()) -> Self {
        Self::new()
    }
}
