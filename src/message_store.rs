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

    pub async fn write_messages(
        &self,
        stream_name: &str,
    ) -> Result<WriteMessages<PoolConnection<Postgres>>, Box<dyn Error>> {
        let conn = self.store.acquire().await?;
        Ok(WriteMessages::new(conn, stream_name))
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

pub struct WriteMessages<Conn> {
    conn: Conn,
    stream_name: String,
    expected_version: Option<i64>,
    messages: Vec<serde_json::Value>,
}

impl<Conn> WriteMessages<Conn> {
    pub fn new(conn: Conn, stream_name: &str) -> Self {
        Self {
            conn,
            stream_name: stream_name.to_owned(),
            expected_version: None,
            messages: Vec::new(),
        }
    }

    pub fn expected_version(mut self, expected_version: i64) -> Self {
        self.expected_version = Some(expected_version);
        self
    }

    pub fn with_message<T>(mut self, message: T) -> Self 
    where
        T: serde::Serialize + crate::Message + Into<crate::Msg<T>>
    {
        let msg = message.into();
        let message_data = serde_json::json!({
            "type": T::TYPE_NAME,
            "data": serde_json::to_value(msg.data).unwrap(),
            "metadata": serde_json::to_value(msg.metadata).unwrap(),
        });

        self.messages.push(message_data);
        self
    }

    pub fn with_batch<T>(mut self, batch: impl AsRef<[T]>) -> Self
    where
        T: serde::Serialize + crate::Message + Clone + Into<crate::Msg<T>>,
    {
        for message in batch.as_ref().iter() {
            let msg = message.clone().into(); 
            self = self.with_message(msg);
        }

        self
    }
}

impl WriteMessages<sqlx::pool::PoolConnection<Postgres>> {
    pub async fn execute(&mut self) -> Result<i64, Box<dyn Error>> {
        use sqlx::Acquire;

        #[derive(sqlx::FromRow)]
        struct LastPosition(i64);
        let mut last_position = LastPosition(-1);

        let mut transaction = self.conn.begin().await?;

        for message in self.messages.iter() {
            let id = uuid::Uuid::new_v4().to_string();

            last_position = sqlx::query_as(
                "SELECT write_message($1::varchar, $2::varchar, $3::varchar, $4::jsonb, $5::jsonb, $6::bigint);",
            )
                .bind(id)
                .bind(&self.stream_name)
                .bind(message.get("type"))
                .bind(message.get("data"))
                .bind(message.get("metadata"))
                .bind(&self.expected_version)
                .fetch_one(&mut *transaction)
                .await?;

            self.expected_version = self.expected_version.map(|version| version + 1);
        }

        transaction.commit().await?;

        Ok(last_position.0)
    }
}
