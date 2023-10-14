use account_component::consumers::commands::*;
use aqueous::{Component, Connection};
use std::error::Error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("account_component").add_directive("ignored".parse()?))
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Set global default subscriber");

    let connection =
        Connection::connect("postgres://message_store@localhost/message_store").await?;

    Component::default()
        .with_connection(connection)
        .add_consumer(CommandsConsumer::build())
        .add_consumer(TransactionsConsumer::build())
        .start()
        .await;

    Ok(())
}
