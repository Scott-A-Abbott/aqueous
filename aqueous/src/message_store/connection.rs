use crate::message_store::Error;
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
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
    max_connections: Option<u32>,
    min_connections: Option<u32>,
    url: Option<String>,
}

impl ConnectionBuilder {
    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    pub fn database(mut self, database: &str) -> Self {
        self.database = Some(database.to_string());
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = Some(min);
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub async fn connect(self) -> Result<Connection, Error> {
        let connect_options = self.build_connect_options()?;
        let pool_options = self.build_pool_options();

        let pool = pool_options.connect_with(connect_options).await?;
        Ok(Connection::new(pool))
    }

    fn build_connect_options(&self) -> Result<PgConnectOptions, Error> {
        let mut connect_options = if let Some(url) = self.url.as_ref() {
            use std::str::FromStr;
            PgConnectOptions::from_str(url)?
        } else {
            PgConnectOptions::default()
        };

        if let Some(host) = self.host.as_ref() {
            connect_options = connect_options.host(host);
        }

        if let Some(port) = self.port {
            connect_options = connect_options.port(port);
        }

        if let Some(username) = self.username.as_ref() {
            connect_options = connect_options.username(username);
        }

        if let Some(password) = self.password.as_ref() {
            connect_options = connect_options.password(password);
        }

        if let Some(database) = self.database.as_ref() {
            connect_options = connect_options.database(database);
        }

        Ok(connect_options)
    }

    fn build_pool_options(&self) -> PgPoolOptions {
        let mut pool_options = PgPoolOptions::default();

        if let Some(max) = self.max_connections {
            pool_options = pool_options.max_connections(max);
        }

        if let Some(min) = self.min_connections {
            pool_options = pool_options.min_connections(min);
        }

        pool_options
    }
}

impl std::ops::Deref for Connection {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
