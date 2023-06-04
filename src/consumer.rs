use crate::{FunctionHandler, GetCategoryMessages, Handler, IntoHandler, MessageData};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::{interval, Duration, Interval, MissedTickBehavior},
};

pub struct Consumer<Executor> {
    handlers: Vec<Box<dyn Handler<Executor>>>,
    identifier: Option<String>,
    position_update_interval: u64,
    position_update_counter: u64,
    batch_size: i64,
    poll_interval_millis: u64,
    category: String,
    executor: Executor,
}

impl<Executor: Clone> Consumer<Executor> {
    pub fn new(executor: Executor, category: &str) -> Self {
        Self {
            handlers: Vec::new(),
            identifier: None,
            position_update_interval: 100,
            position_update_counter: 0,
            batch_size: 1000,
            poll_interval_millis: 100,
            category: category.to_string(),
            executor,
        }
    }

    pub fn add_handler<Params, Return, Func, H>(mut self, handler: H) -> Self
    where
        Executor: Clone + 'static,
        Params: 'static,
        Return: 'static,
        Func: IntoHandler<Executor, Params, Return, Func> + 'static,
        H: IntoHandler<Executor, Params, Return, Func> + 'static,
        FunctionHandler<Executor, Params, Return, Func>: Handler<Executor>,
    {
        let handler: FunctionHandler<Executor, Params, Return, Func> = handler.into_handler();

        let boxed_handler: Box<dyn Handler<Executor>> = Box::new(handler);
        self.handlers.push(boxed_handler);

        self
    }

    pub fn batch_size(mut self, batch_size: i64) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub fn identifier(mut self, identifier: &str) -> Self {
        self.identifier = Some(identifier.to_owned());
        self
    }

    pub fn poll_interval(mut self, poll_interval_millis: u64) -> Self {
        self.poll_interval_millis = poll_interval_millis;
        self
    }

    pub async fn dispatch(&mut self, message_data: MessageData) {
        for handler in self.handlers.iter_mut() {
            let message_data = message_data.clone();
            let executor = self.executor.clone();

            handler.call(message_data, executor);
        }
    }
}

impl Consumer<sqlx::PgPool> {
    pub async fn start(mut self) {
        let batch_size = self.batch_size;
        let poll_interval_millis = self.poll_interval_millis;
        let category = self.category.clone();
        let executor = self.executor.clone();
        let (dispatch_channel, mut dispatch_receiver) = channel::<MessageData>(batch_size as usize);

        tokio::task::spawn(async move {
            Subscribtion::new(
                executor,
                category.as_ref(),
                batch_size,
                poll_interval_millis,
                dispatch_channel,
            )
            .start()
            .await
        });

        loop {
            if let Some(message_data) = dispatch_receiver.recv().await {
                self.dispatch(message_data).await;
            }
        }
    }
}

pub struct Subscribtion<Executor> {
    get: GetCategoryMessages<Executor>,
    poll_interval: Interval,
    position: Option<i64>,
    dispatch_channel: Sender<MessageData>,
}

impl<Executor: Clone> Subscribtion<Executor> {
    pub fn new(
        executor: Executor,
        category: &str,
        batch_size: i64,
        poll_interval_millis: u64,
        dispatch_channel: Sender<MessageData>,
    ) -> Self {
        let poll_interval = Self::interval_from_millis(poll_interval_millis);

        let mut get = GetCategoryMessages::new(executor.clone(), category);
        get.batch_size(batch_size);

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

impl Subscribtion<sqlx::PgPool> {
    pub async fn start(mut self) {
        loop {
            self.poll_interval.tick().await;

            if let Some(position) = self.position {
                self.get.position(position);
            }

            let messages = self.get.execute().await.unwrap();

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
