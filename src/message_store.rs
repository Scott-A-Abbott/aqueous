use crate::MessageData;
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres};
use std::error::Error;

pub struct MessageStore<Store> {
    store: Store,
}

pub type MessageStorePg = MessageStore<PgPool>;
impl MessageStore<PgPool> {
    pub async fn new(url: &str, max_connections: u32) -> Result<Self, Box<dyn Error>> {
        let store = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(url)
            .await?;

        Ok(Self { store })
    }

    pub async fn get_stream_messages(
        &self,
        stream_name: &str,
    ) -> Result<GetStreamMessages<sqlx::pool::PoolConnection<Postgres>>, Box<dyn Error>> {
        let conn = self.store.acquire().await?;
        Ok(GetStreamMessages::new(conn, stream_name))
    }
}

pub struct GetStreamMessages<Conn> {
    conn: Conn,
    stream_name: String,
    position: Option<i64>,
    batch_size: Option<i64>,
    condition: Option<String>,
}

impl<Conn> GetStreamMessages<Conn> {
    pub fn new(conn: Conn, stream_name: &str) -> Self {
        Self {
            conn,
            stream_name: stream_name.to_owned(),
            position: None,
            batch_size: None,
            condition: None,
        }
    }

    pub fn position(mut self, position: i64) -> Self {
        self.position = Some(position);
        self
    }

    pub fn batch_size(mut self, batch_size: i64) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_owned());
        self
    }
}

impl GetStreamMessages<sqlx::pool::PoolConnection<Postgres>> {
    pub async fn execute(&mut self) -> Result<Vec<MessageData>, Box<dyn Error>> {
        sqlx::query_as(
            "SELECT * from get_stream_messages($1::varchar, $2::bigint, $3::bigint, $4::varchar);",
        )
        .bind(&self.stream_name)
        .bind(self.position.unwrap_or_else(|| 0))
        .bind(self.batch_size.unwrap_or_else(|| 1000))
        .bind(&self.condition)
        .fetch_all(&mut self.conn)
        .await
        .map_err(|e| e.into())
    }
}
