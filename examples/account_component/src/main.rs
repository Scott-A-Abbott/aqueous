use account_component::*;
use aqueous::{CategoryType, Component, Consumer};
use sqlx::postgres::PgPoolOptions;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    let AccountCommandCategory(category) = crate::AccountCommandCategory::new();

    Component::default()
        .add_consumer(
            Consumer::new(pool, category)
                .identifier(CategoryType::new("someIdentifier"))
                .add_handlers((
                    handlers::commands::handle_open,
                    handlers::commands::handle_deposit,
                )),
        )
        .start()
        .await;

    Ok(())
}
