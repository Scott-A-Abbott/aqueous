mod account;
mod messages;
mod store;

use account::Account;

use aqueous::{Category, HandlerParam};
use sqlx::PgPool;

pub struct AccountCategory(pub Category);

impl AccountCategory {
    pub fn new() -> Self {
        let category = Category::new("someAccountCategory");
        Self(category)
    }
}

impl HandlerParam<PgPool, ()> for AccountCategory {
    fn build(_: PgPool, _: ()) -> Self {
        Self::new()
    }
}

pub struct AccountCommandCategory(pub Category);

impl AccountCommandCategory {
    pub fn new() -> Self {
        let category = Category::new_command("someAccountCategory");
        Self(category)
    }
}

impl HandlerParam<PgPool, ()> for AccountCommandCategory {
    fn build(_: PgPool, _: ()) -> Self {
        Self::new()
    }
}
