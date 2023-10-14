use crate::*;
use error::Error;
use sqlx::Execute;
use tracing::{debug, instrument, trace};

pub struct GetVersion {
    connection: Connection,
}

impl GetVersion {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    #[instrument(name = "GetVersion::execute", skip(self), fields(%stream_name))]
    pub async fn execute(&self, stream_name: StreamName) -> Result<Version, Error> {
        let StreamName(stream_name) = stream_name;

        #[derive(sqlx::FromRow)]
        struct StreamVersion {
            stream_version: Option<i64>,
        }

        let query =
            sqlx::query_as("SELECT * FROM stream_version($1::varchar)").bind(stream_name.clone());

        trace!("{} [{}]", query.sql(), stream_name);

        let StreamVersion { stream_version } = query.fetch_one(&self.connection.pool).await?;

        let version = stream_version.map_or(Version::initial(), |v| Version(v));
        debug!("Version of stream {} is {}", stream_name, version);

        Ok(version)
    }
}

impl<Settings> HandlerParam<Settings> for GetVersion {
    fn build(connection: Connection, _: Settings) -> Self {
        Self::new(connection)
    }
}
