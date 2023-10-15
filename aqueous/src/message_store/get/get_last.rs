use crate::{
    message_store::{Connection, Error},
    HandlerParam, MessageData,
    stream_name::StreamName
};
use sqlx::Execute;
use tracing::{instrument, trace};

pub struct GetLast {
    connection: Connection,
    message_type: Option<String>,
}

impl GetLast {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            message_type: None,
        }
    }

    pub fn message_type(&mut self, message_type: &str) -> &mut Self {
        self.message_type = Some(message_type.to_string());
        self
    }

    #[instrument(name = "GetLastStreamMessage::execute", skip(self), fields(%stream_name))]
    pub async fn execute(&mut self, stream_name: StreamName) -> Result<Option<MessageData>, Error> {
        let StreamName(stream_name) = stream_name;

        let query =
            sqlx::query_as("SELECT * from get_last_stream_message($1::varchar, $2::varchar);")
                .bind(stream_name.clone())
                .bind(&self.message_type);

        trace!("{} [{}, {:?}]", query.sql(), stream_name, self.message_type);

        query
            .fetch_optional(&self.connection.pool)
            .await
            .map_err(|e| e.into())
    }
}

impl<Settings> HandlerParam<Settings> for GetLast {
    fn build(connection: Connection, _: Settings) -> Self {
        Self::new(connection)
    }
}
