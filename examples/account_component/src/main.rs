use aqueous::{ Component, Consumer, Category, CategoryType };
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    let category = Category::new_command("someAccountCategory");

    Component::default()
        .add_consumer(
            Consumer::new(pool.clone(), category.clone())
                .identifier(CategoryType::new("someIdentifier"))
                .add_handler(account_component::handler),
        )
        .start()
        .await;

    Ok(())
}
