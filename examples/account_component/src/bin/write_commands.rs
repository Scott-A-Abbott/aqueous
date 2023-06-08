use account_component::messages::commands::Deposit;
use aqueous::*;
use std::error::Error;
use time::OffsetDateTime;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    let amount = 10;

    let deposit = Deposit {
        deposit_id: Uuid::new_v4(),
        account_id: Uuid::new_v4(),
        time: OffsetDateTime::now_utc(),
        amount,
    };

    let batch = (0..10)
        .into_iter()
        .map(|_| deposit.clone())
        .collect::<Vec<_>>();

    let category = Category::new_command("someAccountCategory");

    let stream_id = StreamID::new(deposit.account_id);
    let stream_name = category.stream_name(stream_id);

    WriteMessages::new(pool.clone())
        .with_batch(&batch)
        .execute(stream_name)
        .await?;

    println!("Wrote 10 Deposit commands with amount {}", amount);

    Ok(())
}
