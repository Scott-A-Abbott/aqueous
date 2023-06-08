use account_component::*;
use aqueous::{CategoryType, Component, Consumer, IntoHandler};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    let AccountCommandCategory(category) = crate::AccountCommandCategory::new();

    Component::default()
        .add_consumer(
            Consumer::<PgPool, ()>::new(pool.clone(), category.clone())
                .identifier(CategoryType::new("someIdentifier"))
                .add_handler(handlers::commands::handle_open)
                .add_handler(handlers::commands::handle_deposit),
        )
        .start()
        .await;

    Ok(())
}
