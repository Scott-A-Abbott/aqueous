use account_component::{messages::commands::Deposit, AccountCommandCategory};
use aqueous::{
    message_store::{Connection, Write},
    stream_name::StreamID,
};
use std::{collections::VecDeque, env, error::Error, str::FromStr};
use time::OffsetDateTime;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::builder()
        .url("postgres://message_store@localhost/message_store")
        .connect()
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

    let deposit_id = args
        .pop_front()
        .and_then(|arg| Uuid::from_str(&arg).ok())
        .unwrap_or_else(|| Uuid::new_v4());

    let deposit = Deposit {
        account_id,
        amount,
        deposit_id,
        time: OffsetDateTime::now_utc(),
    };

    let AccountCommandCategory(category) = AccountCommandCategory::new();

    let stream_id = StreamID::new(deposit.account_id);
    let stream_name = category.stream_name(stream_id);

    println!("Writing {:#?}", deposit);

    Write::build(connection)
        .add_message(deposit)
        .execute(stream_name)
        .await?;

    Ok(())
}
