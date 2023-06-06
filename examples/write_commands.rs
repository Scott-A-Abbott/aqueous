use aqueous::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Deposit {
    account_id: Uuid,
    amount: i64,
    time: OffsetDateTime,
}
impl Message for Deposit {
    const TYPE_NAME: &'static str = "Deposit";
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect("postgres://message_store@localhost/message_store")
        .await?;

    let account_id = Uuid::new_v4();
    let deposit = Deposit {
        account_id,
        amount: 10,
        time: OffsetDateTime::now_utc(),
    };

    let batch = (0..10)
        .into_iter()
        .map(|_| deposit.clone())
        .collect::<Vec<_>>();

    let stream_name = format!("someAccountCategory:command-{}", deposit.account_id);

    WriteMessages::new(pool.clone())
        .with_batch(&batch)
        .execute(&stream_name)
        .await?;

    println!("Wrote 10 Deposit commands");

    Ok(())
}
