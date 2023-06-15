use crate::*;
use sqlx::{types::Json, PgPool};
use sqlx::{Error as SqlxError, Execute};
use thiserror::Error;
use tracing::{debug, instrument, trace};

#[derive(Error, Debug)]
pub enum MessageStoreError {
    #[error("{0}")]
    WrongExpectedVersion(String),
    #[error("{0}")]
    Database(String),
    #[error("{0}")]
    Pool(String),
    #[error(transparent)]
    Other(SqlxError),
}

impl From<SqlxError> for MessageStoreError {
    fn from(sqlx_error: SqlxError) -> Self {
        match sqlx_error {
            SqlxError::Database(ref error) => {
                let message = error.message();
                if message.contains("Wrong expected version") {
                    return Self::WrongExpectedVersion(message.to_string());
                }

                Self::Database(format!("{}", sqlx_error))
            }
            SqlxError::PoolClosed | SqlxError::PoolTimedOut => {
                Self::Pool(format!("{}", sqlx_error))
            }
            _ => Self::Other(sqlx_error),
        }
    }
}

pub struct GetStreamVersion {
    pool: PgPool,
}

impl GetStreamVersion {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(name = "GetStreamVersion::execute", skip(self), fields(%stream_name))]
    pub async fn execute(&self, stream_name: StreamName) -> Result<Version, MessageStoreError> {
        let StreamName(stream_name) = stream_name;

        #[derive(sqlx::FromRow)]
        struct StreamVersion {
            stream_version: Option<i64>,
        }

        let query =
            sqlx::query_as("SELECT * FROM stream_version($1::varchar)").bind(stream_name.clone());

        trace!("{} [{}]", query.sql(), stream_name);

        let StreamVersion { stream_version } = query.fetch_one(&self.pool).await?;

        let version = stream_version.map_or(Version::initial(), |v| Version(v));
        debug!("Version of stream {} is {}", stream_name, version);

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

    #[instrument(name = "GetLastStreamMessage::execute", skip(self), fields(%stream_name))]
    pub async fn execute(
        &mut self,
        stream_name: StreamName,
    ) -> Result<Option<MessageData>, MessageStoreError> {
        let StreamName(stream_name) = stream_name;

        let query =
            sqlx::query_as("SELECT * from get_last_stream_message($1::varchar, $2::varchar);")
                .bind(stream_name.clone())
                .bind(&self.message_type);

        trace!("{} [{}, {:?}]", query.sql(), stream_name, self.message_type);

        query.fetch_optional(&self.pool).await.map_err(|e| e.into())
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

    #[instrument(name = "GetStreamMessages::execute", skip(self), fields(%stream_name))]
    pub async fn execute(
        &mut self,
        stream_name: StreamName,
    ) -> Result<Vec<MessageData>, MessageStoreError> {
        let StreamName(stream_name) = stream_name;
        let position = self.position.unwrap_or_else(|| 0);
        let batch_size = self.batch_size.unwrap_or_else(|| 1000);

        let query = sqlx::query_as(
            "SELECT * from get_stream_messages($1::varchar, $2::bigint, $3::bigint, $4::varchar);",
        )
        .bind(stream_name.clone())
        .bind(position)
        .bind(batch_size)
        .bind(&self.condition);

        trace!(
            "{} [{}, {}, {:?}]",
            query.sql(),
            stream_name,
            position,
            batch_size
        );

        query.fetch_all(&self.pool).await.map_err(|e| e.into())
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

    #[instrument(name = "GetCategoryMessages::execute", skip(self), fields(%category))]
    pub async fn execute(&self, category: Category) -> Result<Vec<MessageData>, MessageStoreError> {
        let Category(category) = category;
        let position = self.position.unwrap_or_else(|| 0);
        let batch_size = self.batch_size.unwrap_or_else(|| 1000);

        let query = sqlx::query_as(
            "SELECT * FROM get_category_messages($1::varchar, $2::bigint, $3::bigint, $4::varchar, $5::bigint, $6::bigint, $7::varchar);",
        )
        .bind(category.clone())
        .bind(position)
        .bind(batch_size)
        .bind(&self.correlation)
        .bind(&self.consumer_group_member)
        .bind(&self.consumer_group_size)
        .bind(&self.condition);

        trace!(
            "{} [{}, {}, {}, {:?}, {:?}, {:?}, {:?}]",
            query.sql(),
            category,
            position,
            batch_size,
            self.correlation,
            self.consumer_group_member,
            self.consumer_group_size,
            self.condition
        );

        query.fetch_all(&self.pool).await.map_err(|e| e.into())
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

#[derive(Debug, serde::Serialize)]
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

    pub fn add_message<T>(&mut self, message: impl Into<Msg<T>>) -> &mut Self
    where
        T: serde::Serialize + Message,
    {
        let msg: Msg<T> = message.into();

        let data = serde_json::to_string(&msg.data).unwrap();

        let message_data = WriteMessageData {
            data,
            type_name: T::TYPE_NAME.to_owned(),
            metadata: msg.metadata.map(|metadata| Json(metadata)),
        };

        self.messages.push(message_data);
        self
    }

    #[instrument(name = "WriteMessages::execute", skip(self), fields(%stream_name))]
    pub async fn execute(&mut self, stream_name: StreamName) -> Result<i64, MessageStoreError> {
        #[derive(sqlx::FromRow)]
        struct LastPosition(i64);

        let mut last_position = LastPosition(-1);
        let StreamName(ref stream_name) = stream_name;

        let mut transaction = self.pool.begin().await?;

        for message in self.messages.iter() {
            let id = uuid::Uuid::new_v4();
            let version = self.expected_version.as_ref().map(|Version(value)| value);

            let query = sqlx::query_as(
                "SELECT write_message($1::varchar, $2::varchar, $3::varchar, $4::jsonb, $5::jsonb, $6::bigint);",
            )
                .bind(id.clone())
                .bind(stream_name.clone())
                .bind(&message.type_name)
                .bind(&message.data)
                .bind(&message.metadata)
                .bind(&version);

            trace!(
                "{} [{}, {}, {}, {}, {:?}, {:?}]",
                query.sql(),
                id,
                stream_name,
                message.type_name,
                message.data,
                message.metadata,
                version
            );

            last_position = query.fetch_one(&mut *transaction).await?;

            self.expected_version = self
                .expected_version
                .as_mut()
                .map(|Version(value)| Version(*value + 1));
        }

        transaction.commit().await?;

        let LastPosition(position) = last_position;

        Ok(position)
    }
}

impl<Settings> HandlerParam<Settings> for WriteMessages {
    fn build(pool: PgPool, _: Settings) -> Self {
        Self::new(pool)
    }
}
