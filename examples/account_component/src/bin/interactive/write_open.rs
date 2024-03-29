use account_component::{messages::commands::Open, AccountCommandCategory};
use aqueous::{
    message_store::{Connection, Write},
    stream_name::StreamID,
};
use std::{collections::VecDeque, env, error::Error, str::FromStr};
use time::OffsetDateTime;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let message_store_url = dotenvy::var("MESSAGE_STORE_URL")?;
    let connection = Connection::builder()
        .url(&message_store_url)
        .connect()
        .await?;

    let mut args: VecDeque<String> = env::args().collect();
    args.pop_front();

    let account_id = args
        .pop_front()
        .and_then(|arg| Uuid::from_str(&arg).ok())
        .unwrap_or_else(|| Uuid::new_v4());

    let open = Open {
        account_id,
        customer_id: Uuid::new_v4(),
        time: OffsetDateTime::now_utc(),
    };

    let AccountCommandCategory(category) = AccountCommandCategory::new();

    let stream_id = StreamID::new(open.account_id);
    let stream_name = category.stream_name(stream_id);

    println!("Writing {:#?}", open);

    Write::build(connection)
        .add_message(open)
        .execute(stream_name)
        .await?;

    Ok(())
}
