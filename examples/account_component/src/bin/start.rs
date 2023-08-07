use account_component::consumers::commands::*;
use aqueous::{Component, PgPoolOptions};
use std::error::Error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("account_component").add_directive("ignored".parse()?))
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Set global default subscriber");

    let pool = PgPoolOptions::new()
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    Component::default()
        .add_consumer(CommandsConsumer::build())
        .add_consumer(TransactionsConsumer::build())
        .with_pool(pool)
        .start()
        .await;

    Ok(())
}
