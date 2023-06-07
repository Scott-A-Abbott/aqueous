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

        while let Some(res) = set.join_next().await {
            res.unwrap();
        }
    }
}

pub trait Start {
    fn start(&mut self);
}
