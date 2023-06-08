use account_component::consumers::commands::*;
use aqueous::Component;
use sqlx::postgres::PgPoolOptions;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    Component::default()
        .add_consumer(CommandsConsumer::build(pool.clone()))
        .add_consumer(TransactionsConsumer::build(pool.clone()))
        .start()
        .await;

    Ok(())
}
