use crate::Consumer;
use sqlx::{postgres::PgConnectOptions, postgres::PgPoolOptions, PgPool};
use tokio::task::JoinSet;

pub struct Component<Settings = ()> {
    consumers: Vec<Consumer<Settings>>,
    settings: Settings,
    connect_options: PgConnectOptions,
    pool_options: PgPoolOptions,
    pool: Option<PgPool>,
}

impl<S> Default for Component<S>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            consumers: Vec::new(),
            settings: Default::default(),
            connect_options: PgConnectOptions::new(),
            pool_options: PgPoolOptions::new(),
            pool: None,
        }
    }
}

impl<S> Component<S> {
    pub fn new(settings: S) -> Self {
        Self {
            consumers: Vec::new(),
            settings,
            connect_options: PgConnectOptions::new(),
            pool_options: PgPoolOptions::new(),
            pool: None,
        }
    }

    pub fn with_connect_options(mut self, connect_options: PgConnectOptions) -> Self {
        self.connect_options = connect_options;
        self
    }

    pub fn with_pool_options(mut self, pool_options: PgPoolOptions) -> Self {
        self.pool_options = pool_options;
        self
    }

    pub fn with_pool(mut self, pool: PgPool) -> Self {
        self.pool = Some(pool);
        self
    }

    pub fn add_consumer(mut self, consumer: Consumer<S>) -> Self {
        self.consumers.push(consumer);
        self
    }

    pub async fn start(self)
    where
        S: Clone + Send + 'static,
    {
        let mut set = JoinSet::new();
        let pool = match self.pool {
            Some(pool) => pool,
            None => self
                .pool_options
                .connect_with(self.connect_options)
                .await
                .unwrap(),
        };

        for mut consumer in self.consumers {
            let settings = self.settings.clone();
            let pool = pool.clone();

            set.spawn(async move { consumer.start(pool, settings).await });
        }

        // ## Should the entire component exit at any panic? Or just the consumer thread?
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            previous_hook(info);
            std::process::exit(1);
        }));

        set.join_next().await;
    }
}
