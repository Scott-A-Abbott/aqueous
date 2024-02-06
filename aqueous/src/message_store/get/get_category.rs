use crate::{
    message_store::{Connection, Error},
    stream_name::Category,
    HandlerParam, MessageData,
};
use sqlx::Execute;
use tracing::{instrument, trace};

pub struct GetCategory {
    connection: Connection,
    position: Option<i64>,
    batch_size: Option<i64>,
    correlation: Option<String>,
    consumer_group_member: Option<i64>,
    consumer_group_size: Option<i64>,
    condition: Option<String>,
}

impl GetCategory {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
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

    #[instrument(name = "GetCategory::execute", skip(self), fields(%category))]
    pub async fn execute(&self, category: Category) -> Result<Vec<MessageData>, Error> {
        let Category(category) = category;
        let position = self.position.unwrap_or(0);
        let batch_size = self.batch_size.unwrap_or(1000);

        let query = sqlx::query_as(
            "SELECT * FROM get_category_messages($1::varchar, $2::bigint, $3::bigint, $4::varchar, $5::bigint, $6::bigint, $7::varchar);",
        )
        .bind(category.clone())
        .bind(position)
        .bind(batch_size)
        .bind(&self.correlation)
        .bind(self.consumer_group_member)
        .bind(self.consumer_group_size)
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

        query
            .fetch_all(&self.connection.pool)
            .await
            .map_err(|e| e.into())
    }
}

impl<Settings> HandlerParam<Settings> for GetCategory {
    fn build(connection: Connection, _: Settings) -> Self {
        Self::new(connection)
    }
}
