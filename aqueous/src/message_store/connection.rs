use crate::*;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

#[derive(Clone, Debug)]
pub struct Connection {
    pub pool: PgPool,
}

impl Connection {
    pub fn builder() -> ConnectionBuilder {
        ConnectionBuilder::default()
    }

    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Default)]
pub struct ConnectionBuilder {
    connect_options: PgConnectOptions,
    pool_options: PgPoolOptions,
}

impl ConnectionBuilder {
    pub fn host(mut self, host: &str) -> Self {
        self.connect_options = self.connect_options.host(host);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.connect_options = self.connect_options.port(port);
        self
    }

    pub fn username(mut self, username: &str) -> Self {
        self.connect_options = self.connect_options.username(username);
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.connect_options = self.connect_options.password(password);
        self
    }

    pub fn database(mut self, database: &str) -> Self {
        self.connect_options = self.connect_options.database(database);
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.pool_options = self.pool_options.max_connections(max);
        self
    }

    pub fn min_connections(mut self, min: u32) -> Self {
        self.pool_options = self.pool_options.min_connections(min);
        self
    }

    pub async fn build(self) -> Result<Connection, MessageStoreError> {
        let pool = self.pool_options.connect_with(self.connect_options).await?;
        Ok(Connection::new(pool))
    }

    pub async fn build_with_url(self, url: &str) -> Result<Connection, MessageStoreError> {
        let pool = self.pool_options.connect(url).await?;
        Ok(Connection::new(pool))
    }
}

impl std::ops::Deref for Connection {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
