use crate::MessageData;
use sqlx::{pool::PoolConnection, postgres::PgPoolOptions, PgPool, Postgres};
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

    pub fn from_pool(pool: PgPool) -> Self {
        Self { store: pool }
    }

    pub async fn get_stream_messages(
        &self,
        stream_name: &str,
    ) -> Result<GetStreamMessages<PoolConnection<Postgres>>, Box<dyn Error>> {
        let conn = self.store.acquire().await?;
        Ok(GetStreamMessages::new(conn, stream_name))
    }

    pub async fn get_category_messages(
        &self,
        category_name: &str,
    ) -> Result<GetCategoryMessages<PoolConnection<Postgres>>, Box<dyn Error>> {
        let conn = self.store.acquire().await?;
        Ok(GetCategoryMessages::new(conn, category_name))
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

pub struct GetCategoryMessages<Conn> {
    conn: Conn,
    category_name: String,
    position: Option<i64>,
    batch_size: Option<i64>,
    correlation: Option<String>,
    consumer_group_member: Option<i64>,
    consumer_group_size: Option<i64>,
    condition: Option<String>,
}

impl<Conn> GetCategoryMessages<Conn> {
    pub fn new(conn: Conn, stream_name: &str) -> Self {
        Self {
            conn,
            category_name: stream_name.to_owned(),
            position: None,
            batch_size: None,
            correlation: None,
            consumer_group_member: None,
            consumer_group_size: None,
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

    pub fn correlation(mut self, correlation: &str) -> Self {
        self.correlation = Some(correlation.to_owned());
        self
    }

    pub fn consumer_group_member(mut self, consumer_group_member: i64) -> Self {
        self.consumer_group_member = Some(consumer_group_member);
        self
    }

    pub fn consumer_group_size(mut self, consumer_group_size: i64) -> Self {
        self.consumer_group_size = Some(consumer_group_size);
        self
    }

    pub fn condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_owned());
        self
    }
}

impl GetCategoryMessages<sqlx::pool::PoolConnection<Postgres>> {
    pub async fn execute(&mut self) -> Result<Vec<MessageData>, Box<dyn Error>> {
        sqlx::query_as(
            "SELECT * FROM get_category_messages($1::varchar, $2::bigint, $3::bigint, $4::varchar, $5::bigint, $6::bigint, $7::varchar);",
        )
        .bind(&self.category_name)
        .bind(self.position.unwrap_or_else(|| 0))
        .bind(self.batch_size.unwrap_or_else(|| 1000))
        .bind(&self.correlation)
        .bind(&self.consumer_group_member)
        .bind(&self.consumer_group_size)
        .bind(&self.condition)
        .fetch_all(&mut self.conn)
        .await
        .map_err(|e| e.into())
    }
}
