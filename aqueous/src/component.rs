use crate::{Connection, Consumer};
use tokio::task::JoinSet;

pub struct Component<Settings = ()> {
    consumers: Vec<Consumer<Settings>>,
    settings: Settings,
    connection: Connection,
}

impl Component {
    pub fn simple(connection: Connection) -> Self {
        Self {
            consumers: Vec::new(),
            settings: (),
            connection,
        }
    }
}

impl<S> Component<S> {
    pub fn new(connection: Connection, settings: S) -> Self {
        Self {
            consumers: Vec::new(),
            settings,
            connection,
        }
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

        for mut consumer in self.consumers {
            let settings = self.settings.clone();
            let connection = self.connection.clone();

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
