use crate::*;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{collections::HashMap, marker::PhantomData};
use thiserror::Error;
use tokio::{
    sync::mpsc::{channel, Sender},
    time::{interval, Duration, Interval, MissedTickBehavior},
};

#[derive(Error, Debug)]
#[error("A handler that recieves {0} already exists")]
pub struct DuplicateHandlerError(String);

pub struct Consumer<Settings = ()> {
    handlers: HashMap<String, Box<dyn Handler<Settings> + Send>>,
    position_update_interval: u64,
    position_update_counter: u64,
    batch_size: i64,
    poll_interval_millis: u64,
    strict: bool,
    settings_marker: PhantomData<Settings>,
    category: Category,
    catchall: Option<Box<dyn Handler<Settings> + Send>>,
    identifier: Option<CategoryType>,
    correlation: Option<String>,
    group: Option<(i64, i64)>,
}

impl<Settings> Consumer<Settings> {
    pub fn new(category: Category) -> Self
    where
        Settings: Default,
    {
        Self {
            category,
            handlers: HashMap::new(),
            position_update_interval: 100,
            position_update_counter: 0,
            batch_size: 1000,
            poll_interval_millis: 100,
            strict: false,
            settings_marker: Default::default(),
            catchall: None,
            identifier: None,
            correlation: None,
            group: None,
        }
    }

    pub fn insert_handler<Params, Return, Func>(
        mut self,
        handler: Func,
    ) -> Result<Self, DuplicateHandlerError>
    where
        Params: Send + 'static,
        Return: Send + 'static,
        Func: IntoHandler<Params, Return, Func> + Send + 'static,
        FunctionHandler<Params, Return, Func>: Handler<Settings>,
    {
        let handler: FunctionHandler<Params, Return, Func> = handler.into_handler();
        let message_type = handler.message_type();

        if self.handlers.contains_key(&message_type) {
            return Err(DuplicateHandlerError(message_type));
        }

        let boxed_handler: Box<dyn Handler<Settings> + Send> = Box::new(handler);
        self.handlers.insert(message_type, boxed_handler);

        Ok(self)
    }

    pub fn extend_handlers<Params, Return>(
        mut self,
        handlers: impl IntoHandlerCollection<Params, Return, Settings>,
    ) -> Result<Self, DuplicateHandlerError> {
        let HandlerCollection { handlers, .. } = handlers.into_handler_collection();

        for (message_type, handler) in handlers.into_iter() {
            if self.handlers.contains_key(&message_type) {
                return Err(DuplicateHandlerError(message_type));
            }

            self.handlers.insert(message_type, handler);
        }

        Ok(self)
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

    pub fn group(mut self, size: i64, member: i64) -> Self {
        self.group = Some((size, member));
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

        for handler in self.handlers.values_mut() {
            let message_data = message_data.clone();
            let settings = settings.clone();

            let was_processed = handler.call(message_data, pool.clone(), settings);
            processed_message = processed_message || was_processed;
        }

        if let Some(catchall) = self.catchall.as_mut() {
            processed_message = catchall.call(message_data.clone(), pool, settings);
        }

        if self.strict && !processed_message {
            panic!("Did not process message while strict: {:?}", message_data);
        }
    }

    pub async fn start(&mut self, pool: PgPool, settings: Settings)
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

        if let Some((size, member)) = self.group {
            get.consumer_group_size(size).consumer_group_member(member);
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
        write.add_message(Recorded { position });

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
