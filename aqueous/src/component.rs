use crate::{Connection, ConnectionBuilder, Consumer};
use tokio::task::JoinSet;

pub struct Component<Settings = ()> {
    consumers: Vec<Consumer<Settings>>,
    settings: Settings,
    connection: Option<Connection>,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            consumers: Vec::new(),
            settings: (),
            connection: None,
        }
    }
}

impl<S> Component<S> {
    pub fn new(connection: Connection, settings: S) -> Self {
        Self {
            consumers: Vec::new(),
            settings,
            connection: Some(connection),
        }
    }

    pub fn with_connection(mut self, connection: Connection) -> Self {
        self.connection = Some(connection);
        self
    }

    pub fn add_consumer(mut self, consumer: Consumer<S>) -> Self {
        self.consumers.push(consumer);
        self
    }

    pub async fn start(mut self)
    where
        S: Clone + Send + 'static,
    {
        let mut set = JoinSet::new();

        for mut consumer in self.consumers {
            let settings = self.settings.clone();
            let connection = match self.connection.take() {
                Some(connection) => connection,
                None => ConnectionBuilder::default()
                    .build()
                    .await
                    .expect("Connecting with default options"),
            };

            set.spawn(async move { consumer.start(connection, settings).await });
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
