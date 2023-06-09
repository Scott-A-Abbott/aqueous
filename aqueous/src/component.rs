use tokio::task::JoinSet;

pub struct Component {
    consumers: Vec<Box<dyn Start + Send>>,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            consumers: Vec::new(),
        }
    }
}

impl Component {
    pub fn add_consumer<C>(mut self, consumer: C) -> Self
    where
        C: Start + Send + 'static,
    {
        let boxed_consumer: Box<dyn Start + Send> = Box::new(consumer);
        self.consumers.push(boxed_consumer);
        self
    }

    pub async fn start(self) {
        let mut set = JoinSet::new();

        for mut consumer in self.consumers {
            set.spawn_blocking(move || {
                consumer.start();
            });
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

pub trait Start {
    fn start(&mut self);
}
