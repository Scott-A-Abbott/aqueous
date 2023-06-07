mod account;
mod messages;
mod store;

use account::Account;

use aqueous::{HandlerParam, Category};
use sqlx::PgPool;

pub struct AccountCategory(Category);

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

pub struct AccountCommandCategory(Category);

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
