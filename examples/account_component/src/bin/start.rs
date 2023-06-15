use account_component::consumers::commands::*;
use aqueous::{Component, PgPoolOptions};
use std::error::Error;
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::new("account_component").add_directive("ignored".parse()?))
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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
