use crate::*;
use sqlx::{types::Json, Acquire, PgExecutor, Postgres};
use std::error::Error;

pub struct GetStreamVersion<Executor> {
    executor: Executor,
}

impl<Executor> GetStreamVersion<Executor> {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
}

impl<Executor> GetStreamVersion<Executor>
where
    for<'e, 'c> &'e Executor: PgExecutor<'c>,
{
    pub async fn execute(&self, stream_name: StreamName) -> Result<Version, Box<dyn Error>> {
        let StreamName(stream_name) = stream_name;

        #[derive(sqlx::FromRow, serde::Deserialize)]
        struct StreamVersion {
            stream_version: i64,
        }

        let StreamVersion { stream_version } =
            sqlx::query_as("SELECT * FROM stream_version($1::varchar)")
                .bind(stream_name)
                .fetch_one(&self.executor)
                .await?;

        Ok(Version(stream_version))
    }
}

pub struct GetLastStreamMessage<Executor> {
    executor: Executor,
    message_type: Option<String>,
}

impl<Executor> GetLastStreamMessage<Executor> {
    pub fn new(executor: Executor) -> Self {
        Self {
            executor,
            message_type: None,
        }
    }

    pub fn message_type(mut self, message_type: &str) -> Self {
        self.message_type = Some(message_type.to_string());
        self
    }
}

impl<Executor> GetLastStreamMessage<Executor>
where
    for<'e, 'c> &'e Executor: PgExecutor<'c>,
{
    pub async fn execute(
        &mut self,
        stream_name: StreamName,
    ) -> Result<Option<MessageData>, Box<dyn Error>> {
        let StreamName(stream_name) = stream_name;

        sqlx::query_as("SELECT * from get_last_stream_message($1::varchar, $2::varchar);")
            .bind(stream_name)
            .bind(&self.message_type)
            .fetch_optional(&self.executor)
            .await
            .map_err(|e| e.into())
    }
}

pub struct GetStreamMessages<Executor> {
    executor: Executor,
    position: Option<i64>,
    batch_size: Option<i64>,
    condition: Option<String>,
}

impl<Executor> GetStreamMessages<Executor> {
    pub fn new(executor: Executor) -> Self {
        Self {
            executor,
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
}

impl<Executor> GetStreamMessages<Executor>
where
    for<'e, 'c> &'e Executor: PgExecutor<'c>,
{
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
        .fetch_all(&self.executor)
        .await
        .map_err(|e| e.into())
    }
}

pub struct GetCategoryMessages<Executor> {
    executor: Executor,
    category: Category,
    position: Option<i64>,
    batch_size: Option<i64>,
    correlation: Option<String>,
    consumer_group_member: Option<i64>,
    consumer_group_size: Option<i64>,
    condition: Option<String>,
}

impl<Executor> GetCategoryMessages<Executor> {
    pub fn new(executor: Executor, category: Category) -> Self {
        Self {
            executor,
            category,
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
}

impl<Executor> GetCategoryMessages<Executor>
where
    for<'e, 'c> &'e Executor: PgExecutor<'c>,
{
    pub async fn execute(&self) -> Result<Vec<MessageData>, Box<dyn Error>> {
        let Category(ref category) = self.category;

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
        .fetch_all(&self.executor)
        .await
        .map_err(|e| e.into())
    }
}

pub struct WriteMessages<Executor> {
    executor: Executor,
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

impl<Executor> WriteMessages<Executor> {
    pub fn new(executor: Executor) -> Self {
        Self {
            executor,
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

    pub fn with_message<T>(&mut self, message: T) -> &mut Self
    where
        T: serde::Serialize + Message + Into<Msg<T>>,
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
}

impl<Executor> WriteMessages<Executor>
where
    for<'e, 'c> &'e Executor: Acquire<'c, Database = Postgres>,
{
    pub async fn execute(&mut self, stream_name: StreamName) -> Result<i64, Box<dyn Error>> {
        #[derive(sqlx::FromRow)]
        struct LastPosition(i64);

        let mut last_position = LastPosition(-1);
        let StreamName(ref stream_name) = stream_name;

        let mut transaction = self.executor.begin().await?;

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

impl<Executor: Clone + 'static> HandlerParam<Executor> for WriteMessages<Executor> {
    fn build(_: MessageData, executor: Executor) -> Self {
        let writer = Self::new(executor.clone());

        writer
    }
}
