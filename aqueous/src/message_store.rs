use crate::*;
use sqlx::{types::Json, PgPool};
use std::error::Error;

pub struct GetStreamVersion {
    pool: PgPool,
}

impl GetStreamVersion {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn execute(&self, stream_name: StreamName) -> Result<Version, Box<dyn Error>> {
        let StreamName(stream_name) = stream_name;

        #[derive(sqlx::FromRow)]
        struct StreamVersion {
            stream_version: Option<i64>,
        }

        let StreamVersion { stream_version } =
            sqlx::query_as("SELECT * FROM stream_version($1::varchar)")
                .bind(stream_name)
                .fetch_one(&self.pool)
                .await?;

        let version = match stream_version {
            Some(version) => Version(version),
            None => Version::initial(),
        };

        Ok(version)
    }
}

impl<Settings> HandlerParam<Settings> for GetStreamVersion {
    fn build(pool: PgPool, _: Settings) -> Self {
        Self::new(pool)
    }
}

pub struct GetLastStreamMessage {
    pool: PgPool,
    message_type: Option<String>,
}

impl GetLastStreamMessage {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            message_type: None,
        }
    }

    pub fn message_type(mut self, message_type: &str) -> Self {
        self.message_type = Some(message_type.to_string());
        self
    }

    pub async fn execute(
        &mut self,
        stream_name: StreamName,
    ) -> Result<Option<MessageData>, Box<dyn Error>> {
        let StreamName(stream_name) = stream_name;

        sqlx::query_as("SELECT * from get_last_stream_message($1::varchar, $2::varchar);")
            .bind(stream_name)
            .bind(&self.message_type)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.into())
    }
}

impl<Settings> HandlerParam<Settings> for GetLastStreamMessage {
    fn build(pool: PgPool, _: Settings) -> Self {
        Self::new(pool)
    }
}

pub struct GetStreamMessages {
    pool: PgPool,
    position: Option<i64>,
    batch_size: Option<i64>,
    condition: Option<String>,
}

impl GetStreamMessages {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            position: None,
            batch_size: None,
            condition: None,
        }
    }

    pub fn position(&mut self, position: i64) -> &mut Self {
        self.position = Some(position);
        self
    }

    pub fn batch_size(&mut self, batch_size: i64) -> &mut Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn condition(&mut self, condition: &str) -> &mut Self {
        self.condition = Some(condition.to_owned());
        self
    }

    pub async fn execute(
        &mut self,
        stream_name: StreamName,
    ) -> Result<Vec<MessageData>, Box<dyn Error>> {
        let StreamName(stream_name) = stream_name;

        sqlx::query_as(
            "SELECT * from get_stream_messages($1::varchar, $2::bigint, $3::bigint, $4::varchar);",
        )
        .bind(stream_name)
        .bind(self.position.unwrap_or_else(|| 0))
        .bind(self.batch_size.unwrap_or_else(|| 1000))
        .bind(&self.condition)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.into())
    }
}

impl<Settings> HandlerParam<Settings> for GetStreamMessages {
    fn build(pool: PgPool, _: Settings) -> Self {
        Self::new(pool)
    }
}

pub struct GetCategoryMessages {
    pool: PgPool,
    position: Option<i64>,
    batch_size: Option<i64>,
    correlation: Option<String>,
    consumer_group_member: Option<i64>,
    consumer_group_size: Option<i64>,
    condition: Option<String>,
}

impl GetCategoryMessages {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            position: None,
            batch_size: None,
            correlation: None,
            consumer_group_member: None,
            consumer_group_size: None,
            condition: None,
        }
    }

    pub fn position(&mut self, position: i64) -> &mut Self {
        self.position = Some(position);
        self
    }

    pub fn batch_size(&mut self, batch_size: i64) -> &mut Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn correlation(&mut self, correlation: &str) -> &mut Self {
        self.correlation = Some(correlation.to_owned());
        self
    }

    pub fn consumer_group_member(&mut self, consumer_group_member: i64) -> &mut Self {
        self.consumer_group_member = Some(consumer_group_member);
        self
    }

    pub fn consumer_group_size(&mut self, consumer_group_size: i64) -> &mut Self {
        self.consumer_group_size = Some(consumer_group_size);
        self
    }

    pub fn condition(&mut self, condition: &str) -> &mut Self {
        self.condition = Some(condition.to_owned());
        self
    }

    pub async fn execute(&self, category: Category) -> Result<Vec<MessageData>, Box<dyn Error>> {
        let Category(category) = category;

        sqlx::query_as(
            "SELECT * FROM get_category_messages($1::varchar, $2::bigint, $3::bigint, $4::varchar, $5::bigint, $6::bigint, $7::varchar);",
        )
        .bind(category)
        .bind(self.position.unwrap_or_else(|| 0))
        .bind(self.batch_size.unwrap_or_else(|| 1000))
        .bind(&self.correlation)
        .bind(&self.consumer_group_member)
        .bind(&self.consumer_group_size)
        .bind(&self.condition)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.into())
    }
}

impl<Settings> HandlerParam<Settings> for GetCategoryMessages {
    fn build(pool: PgPool, _: Settings) -> Self {
        Self::new(pool)
    }
}

pub struct WriteMessages {
    pool: PgPool,
    expected_version: Option<Version>,
    messages: Vec<WriteMessageData>,
}

#[derive(serde::Serialize)]
struct WriteMessageData {
    #[serde(rename = "type")]
    type_name: String,
    data: String,
    metadata: Option<Json<Metadata>>,
}

impl WriteMessages {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            expected_version: None,
            messages: Vec::new(),
        }
    }

    pub fn expected_version(&mut self, expected_version: Version) -> &mut Self {
        self.expected_version = Some(expected_version);
        self
    }

    pub fn initial(&mut self) -> &mut Self {
        self.expected_version = Some(Version::initial());
        self
    }

    pub fn with_message<T>(&mut self, message: impl Into<Msg<T>>) -> &mut Self
    where
        T: serde::Serialize + Message,
    {
        let msg: Msg<T> = message.into();

        let message_data = WriteMessageData {
            type_name: T::TYPE_NAME.to_owned(),
            data: serde_json::to_string(&msg.data).unwrap(),
            metadata: msg.metadata.map(|metadata| Json(metadata)),
        };

        self.messages.push(message_data);
        self
    }

    pub fn with_batch<T>(&mut self, batch: impl AsRef<[T]>) -> &mut Self
    where
        T: serde::Serialize + Message + Clone + Into<Msg<T>>,
    {
        for message in batch.as_ref().iter() {
            self.with_message(message.clone());
        }

        self
    }

    pub async fn execute(&mut self, stream_name: StreamName) -> Result<i64, Box<dyn Error>> {
        #[derive(sqlx::FromRow)]
        struct LastPosition(i64);

        let mut last_position = LastPosition(-1);
        let StreamName(ref stream_name) = stream_name;

        let mut transaction = self.pool.begin().await?;

        for message in self.messages.iter() {
            let id = uuid::Uuid::new_v4();
            let version = self.expected_version.as_ref().map(|Version(value)| value);

            last_position = sqlx::query_as(
                "SELECT write_message($1::varchar, $2::varchar, $3::varchar, $4::jsonb, $5::jsonb, $6::bigint);",
            )
                .bind(id)
                .bind(stream_name)
                .bind(&message.type_name)
                .bind(&message.data)
                .bind(&message.metadata)
                .bind(version)
                .fetch_one(&mut *transaction)
                .await?;

            self.expected_version = self
                .expected_version
                .as_mut()
                .map(|Version(value)| Version(*value + 1));
        }

        transaction.commit().await?;

        let LastPosition(position) = last_position;

        if position == -1 {
            return Err("No messages were written".into());
        }

        Ok(position)
    }
}

impl<Settings> HandlerParam<Settings> for WriteMessages {
    fn build(pool: PgPool, _: Settings) -> Self {
        Self::new(pool)
    }
}
