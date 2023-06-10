use crate::*;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::marker::PhantomData;
use tokio::{
    sync::mpsc::{channel, Sender},
    time::{interval, Duration, Interval, MissedTickBehavior},
};

pub struct Consumer<Settings = ()> {
    handlers: Vec<Box<dyn Handler<Settings> + Send>>,
    catchall: Option<Box<dyn Handler<Settings> + Send>>,
    identifier: Option<CategoryType>,
    correlation: Option<String>,
    position_update_interval: u64,
    position_update_counter: u64,
    batch_size: i64,
    poll_interval_millis: u64,
    strict: bool,
    settings_marker: PhantomData<Settings>,
    category: Category,
}

impl<Settings> Consumer<Settings> {
    pub fn new(category: Category) -> Self
    where
        Settings: Default,
    {
        Self {
            handlers: Vec::new(),
            catchall: None,
            identifier: None,
            correlation: None,
            position_update_interval: 100,
            position_update_counter: 0,
            batch_size: 1000,
            poll_interval_millis: 100,
            strict: false,
            settings_marker: Default::default(),
            category,
        }
    }

    pub fn add_handler<Params, Return, Func, H>(mut self, handler: H) -> Self
    where
        Params: Send + 'static,
        Return: Send + 'static,
        Func: IntoHandler<Params, Return, Func> + Send + 'static,
        H: IntoHandler<Params, Return, Func> + 'static,
        FunctionHandler<Params, Return, Func>: Handler<Settings>,
    {
        let handler: FunctionHandler<Params, Return, Func> = handler.into_handler();

        let boxed_handler: Box<dyn Handler<Settings> + Send> = Box::new(handler);
        self.handlers.push(boxed_handler);

        self
    }

    pub fn add_handlers<Params, Return>(
        mut self,
        handlers: impl IntoHandlerCollection<Params, Return, Settings>,
    ) -> Self {
        let HandlerCollection { mut handlers, .. } = handlers.into_handler_collection();
        self.handlers.append(&mut handlers);

        self
    }

    pub fn catchall<Params, Return, Func>(mut self, catchall: Func) -> Self
    where
        Params: Send + 'static,
        Return: Send + 'static,
        Func: IntoCatchallHandler<Params, Return, Func> + Send + 'static,
        CatchallFunctionHandler<Params, Return, Func>: Handler<Settings>,
    {
        let catchall_handler = catchall.into_catchall_handler();

        let boxed_handler: Box<dyn Handler<Settings> + Send> = Box::new(catchall_handler); 
        self.catchall = Some(boxed_handler);

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

    pub fn correlation(mut self, correlation: &str) -> Self {
        self.correlation = Some(correlation.to_owned());
        self
    }

    pub fn poll_interval(mut self, poll_interval_millis: u64) -> Self {
        self.poll_interval_millis = poll_interval_millis;
        self
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
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

    pub fn dispatch(&mut self, pool: PgPool, message_data: MessageData, settings: Settings)
    where
        Settings: Clone,
    {
        let mut processed_message = false;

        for handler in self.handlers.iter_mut() {
            let message_data = message_data.clone();
            let settings = settings.clone();

           let was_processed = handler.call(message_data, pool.clone(), settings);
           if was_processed {
               processed_message = was_processed;
           }
        }

        if let Some(catchall) = self.catchall.as_mut() {
            processed_message = catchall.call(message_data.clone(), pool, settings);
        }

        if self.strict && !processed_message {
            panic!("Did not process message while strict: {:?}", message_data);
        }
    }

    pub async fn start(mut self, pool: PgPool, settings: Settings)
    where
        Settings: Clone,
    {
        let mut get = GetCategoryMessages::new(pool.clone());

        if let Some(correlation) = self.correlation.as_ref() {
            get.correlation(correlation);
        }

        if let Some(position) = self.get_position(pool.clone()).await {
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

                self.dispatch(pool.clone(), message_data, settings.clone());
                self.update_position(pool.clone(), update_position).await;
            }
        }
    }

    pub async fn update_position(&mut self, pool: PgPool, position: i64) {
        self.position_update_counter += 1;

        let mut write = WriteMessages::new(pool.clone());
        write.with_message(Recorded { position });

        if self.position_update_counter >= self.position_update_interval {
            let stream_name = self.position_stream_name();
            write.execute(stream_name).await.unwrap();

            self.position_update_counter = 0;
        }
    }

    pub async fn get_position(&mut self, pool: PgPool) -> Option<i64> {
        let stream_name = self.position_stream_name();
        let message_data = GetLastStreamMessage::new(pool.clone())
            .execute(stream_name)
            .await
            .ok()??;

        let recorded = Msg::<Recorded>::from_data(message_data).ok()?;

        Some(recorded.position)
    }
}

impl<S> Start<S> for Consumer<S>
where
    S: Clone,
{
    fn start(&mut self, pool: PgPool, settings: S) {
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                let handlers = self.handlers.drain(..).collect::<Vec<_>>();

                let consumer = Consumer {
                    handlers,
                    catchall: self.catchall.take(),
                    identifier: self.identifier.take(),
                    correlation: self.correlation.take(),
                    position_update_interval: self.position_update_interval,
                    position_update_counter: self.position_update_counter,
                    batch_size: self.batch_size,
                    poll_interval_millis: self.poll_interval_millis,
                    strict: self.strict,
                    settings_marker: Default::default(),
                    category: self.category.clone(),
                };
                Consumer::start(consumer, pool, settings).await;
            })
        });
    }
}

pub struct Subscription {
    get: GetCategoryMessages,
    poll_interval: Interval,
    position: Option<i64>,
    dispatch_channel: Sender<MessageData>,
}

impl Subscription {
    pub fn new(
        get: GetCategoryMessages,
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
