use account_component::{messages::commands::Withdraw, AccountCommandCategory};
use aqueous::*;
use std::{collections::VecDeque, env, error::Error, str::FromStr};
use time::OffsetDateTime;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::builder()
        .build_with_url("postgres://message_store@localhost/message_store")
        .await?;

    let mut args: VecDeque<String> = env::args().collect();
    args.pop_front();

    let account_id = args
        .pop_front()
        .and_then(|arg| Uuid::from_str(&arg).ok())
        .unwrap_or_else(|| Uuid::new_v4());

    let amount = args
        .pop_front()
        .and_then(|arg| i64::from_str(&arg).ok())
        .unwrap_or(10);

    let withdraw = Withdraw {
        account_id,
        amount,
        withdrawal_id: Uuid::new_v4(),
        time: OffsetDateTime::now_utc(),
    };

    let AccountCommandCategory(category) = AccountCommandCategory::new();

    let stream_id = StreamID::new(withdraw.account_id);
    let stream_name = category.stream_name(stream_id);

    println!("Writing {:#?}", withdraw);

    WriteMessages::new(connection)
        .add_message(withdraw)
        .execute(stream_name)
        .await?;

    Ok(())
}
