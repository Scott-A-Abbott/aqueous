use tokio::task::JoinSet;

pub struct Component<Settings = ()> {
    consumers: Vec<Box<dyn Start<Settings> + Send>>,
    settings: Settings,
}

impl<S> Default for Component<S>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            consumers: Vec::new(),
            settings: Default::default(),
        }
    }
}

impl<S> Component<S> {
    pub fn add_consumer<C>(mut self, consumer: C) -> Self
    where
        C: Start<S> + Send + 'static,
    {
        let boxed_consumer: Box<dyn Start<S> + Send> = Box::new(consumer);
        self.consumers.push(boxed_consumer);
        self
    }

    pub async fn start(self)
    where
        S: Clone + Send + 'static,
    {
        let mut set = JoinSet::new();

        for mut consumer in self.consumers {
            let settings = self.settings.clone();

            set.spawn_blocking(move || {
                consumer.start(settings);
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

pub trait Start<S> {
    fn start(&mut self, settings: S);
}
