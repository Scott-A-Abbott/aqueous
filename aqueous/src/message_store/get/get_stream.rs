use crate::{
    message_store::{Connection, Error},
    stream_name::StreamName,
    HandlerParam, MessageData,
};
use sqlx::Execute;
use tracing::{instrument, trace};

pub struct GetStream {
    connection: Connection,
    position: Option<i64>,
    batch_size: Option<i64>,
    condition: Option<String>,
}

impl GetStream {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
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

    #[instrument(name = "GetStream::execute", skip(self), fields(%stream_name))]
    pub async fn execute(&mut self, stream_name: StreamName) -> Result<Vec<MessageData>, Error> {
        let StreamName(stream_name) = stream_name;
        let position = self.position.unwrap_or(0);
        let batch_size = self.batch_size.unwrap_or(1000);

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

        query
            .fetch_all(&self.connection.pool)
            .await
            .map_err(|e| e.into())
    }
}

impl<Settings> HandlerParam<Settings> for GetStream {
    fn build(connection: Connection, _: Settings) -> Self {
        Self::new(connection)
    }
}
