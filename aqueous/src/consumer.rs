use crate::*;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{channel, Sender},
    time::{interval, Duration, Interval, MissedTickBehavior},
};

pub struct Consumer<Executor, Settings> {
    handlers: Vec<Box<dyn Handler<Executor, Settings> + Send>>,
    identifier: Option<CategoryType>,
    correlation: Option<String>,
    position_update_interval: u64,
    position_update_counter: u64,
    batch_size: i64,
    poll_interval_millis: u64,
    settings: Settings,
    category: Category,
    executor: Executor,
}

impl<Executor, Settings> Consumer<Executor, Settings> {
    pub fn new(executor: Executor, category: Category) -> Self
    where
        Settings: Default,
    {
        Self {
            handlers: Vec::new(),
            identifier: None,
            correlation: None,
            position_update_interval: 100,
            position_update_counter: 0,
            batch_size: 1000,
            poll_interval_millis: 100,
            settings: Default::default(),
            category,
            executor,
        }
    }

    pub fn add_handler<Params, Return, Func, H>(mut self, handler: H) -> Self
    where
        Executor: Clone + Send + 'static,
        Params: Send + 'static,
        Return: Send + 'static,
        Func: IntoHandler<Executor, Params, Return, Func> + Send + 'static,
        H: IntoHandler<Executor, Params, Return, Func> + 'static,
        FunctionHandler<Executor, Params, Return, Func>: Handler<Executor, Settings>,
    {
        let handler: FunctionHandler<Executor, Params, Return, Func> = handler.into_handler();

        let boxed_handler: Box<dyn Handler<Executor, Settings> + Send> = Box::new(handler);
        self.handlers.push(boxed_handler);

        self
    }

    pub fn batch_size(mut self, batch_size: i64) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub fn identifier(mut self, identifier: CategoryType) -> Self {
        self.identifier = Some(identifier);
        self
    }

    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }

    pub fn correlation(mut self, correlation: &str) -> Self {
        self.correlation = Some(correlation.to_owned());
        self
    }

    pub fn poll_interval(mut self, poll_interval_millis: u64) -> Self {
        self.poll_interval_millis = poll_interval_millis;
        self
    }

    pub fn position_update_interval(mut self, interval: u64) -> Self {
        self.position_update_interval = interval;
        self
    }

    pub fn position_stream_name(&self) -> StreamName {
        let position_type = CategoryType::new("position");
        let mut category = self.category.add_type(position_type);

        if let Some(identifier) = self.identifier.clone() {
            category = category.add_type(identifier);
        }

        let Category(category) = category;

        StreamName(category)
    }

    pub fn dispatch(&mut self, message_data: MessageData)
    where
        Settings: Clone,
        Executor: Clone,
    {
        for handler in self.handlers.iter_mut() {
            let message_data = message_data.clone();
            let executor = self.executor.clone();
            let settings = self.settings.clone();

            handler.call(message_data, executor, settings);
        }
    }
}

impl<Executor, Settings> Consumer<Executor, Settings>
where
    Executor: Clone + Send + Sync + 'static,
    Settings: Clone,
    for<'e, 'c> &'e Executor: sqlx::Acquire<'c, Database = sqlx::Postgres>,
    for<'e, 'c> &'e Executor: sqlx::PgExecutor<'c>,
{
    pub async fn start(mut self) {
        let mut get = GetCategoryMessages::new(self.executor.clone());

        if let Some(correlation) = self.correlation.as_ref() {
            get.correlation(correlation);
        }

        if let Some(position) = self.get_position().await {
            get.position(position);
        }

        get.batch_size(self.batch_size);

        let category = self.category.clone();
        let poll_interval_millis = self.poll_interval_millis;
        let (dispatch_channel, mut dispatch_receiver) =
            channel::<MessageData>(self.batch_size as usize);

        tokio::task::spawn(async move {
            Subscription::new(get, poll_interval_millis, dispatch_channel)
                .start(category)
                .await
        });

        loop {
            if let Some(message_data) = dispatch_receiver.recv().await {
                let update_position = message_data.global_position;

                self.dispatch(message_data);
                self.update_position(update_position).await;
            }
        }
    }

    pub async fn update_position(&mut self, position: i64) {
        self.position_update_counter += 1;

        let mut write = WriteMessages::new(self.executor.clone());
        write.with_message(Recorded { position });

        if self.position_update_counter >= self.position_update_interval {
            let stream_name = self.position_stream_name();
            write.execute(stream_name).await.unwrap();

            self.position_update_counter = 0;
        }
    }

    pub async fn get_position(&mut self) -> Option<i64> {
        let stream_name = self.position_stream_name();
        let message_data = GetLastStreamMessage::new(self.executor.clone())
            .execute(stream_name)
            .await
            .ok()??;

        let recorded = Msg::<Recorded>::from_data(message_data).ok()?;

        Some(recorded.position)
    }
}

impl<E, S> Start for Consumer<E, S>
where
    E: Clone + Send + Sync + 'static,
    S: Clone,
    for<'e, 'c> &'e E: sqlx::Acquire<'c, Database = sqlx::Postgres>,
    for<'e, 'c> &'e E: sqlx::PgExecutor<'c>,
{
    fn start(&mut self) {
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                let handlers = self.handlers.drain(..).collect::<Vec<_>>();
                let consumer = Consumer {
                    handlers,
                    identifier: self.identifier.clone(),
                    correlation: self.correlation.clone(),
                    position_update_interval: self.position_update_interval.clone(),
                    position_update_counter: self.position_update_counter.clone(),
                    batch_size: self.batch_size.clone(),
                    poll_interval_millis: self.poll_interval_millis.clone(),
                    settings: self.settings.clone(),
                    category: self.category.clone(),
                    executor: self.executor.clone(),
                };
                Consumer::start(consumer).await;
            })
        });
    }
}

pub struct Subscription<Executor> {
    get: GetCategoryMessages<Executor>,
    poll_interval: Interval,
    position: Option<i64>,
    dispatch_channel: Sender<MessageData>,
}

impl<Executor> Subscription<Executor> {
    pub fn new(
        get: GetCategoryMessages<Executor>,
        poll_interval_millis: u64,
        dispatch_channel: Sender<MessageData>,
    ) -> Self {
        let poll_interval = Self::interval_from_millis(poll_interval_millis);

        Self {
            get,
            poll_interval,
            position: None,
            dispatch_channel,
        }
    }

    fn interval_from_millis(millis: u64) -> Interval {
        let duration = Duration::from_millis(millis);
        let mut interval = interval(duration);

        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        interval
    }
}

impl<Executor> Subscription<Executor>
where
    for<'e, 'c> &'e Executor: sqlx::PgExecutor<'c>,
{
    pub async fn start(mut self, category: Category) {
        loop {
            self.poll_interval.tick().await;

            if let Some(position) = self.position {
                self.get.position(position);
            }

            let messages = self.get.execute(category.clone()).await.unwrap();

            for message in messages.into_iter() {
                let message_pos = message.global_position;

                if let Some(pos) = self.position {
                    if pos >= message_pos {
                        continue;
                    }
                }

                self.dispatch_channel.send(message).await.unwrap();
                self.position.replace(message_pos);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Recorded {
    position: i64,
}

impl Message for Recorded {
    const TYPE_NAME: &'static str = "Recorded";
}
