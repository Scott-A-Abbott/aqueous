use crate::{
    FunctionHandler, GetCategoryMessages, GetLastStreamMessage, Handler, IntoHandler, Message,
    MessageData, Msg, WriteMessages,
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{channel, Sender},
    time::{interval, Duration, Interval, MissedTickBehavior},
};

pub struct Consumer<Executor> {
    handlers: Vec<Box<dyn Handler<Executor>>>,
    identifier: Option<String>,
    correlation: Option<String>,
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
            correlation: None,
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

    pub fn position_stream_name(&self) -> String {
        let mut stream_name = match self.category.split(":").collect::<Vec<_>>()[..] {
            [entity_id] => format!("{}:position", entity_id),
            [entity_id, types] => format!("{}:{}+position", entity_id, types),
            _ => self.category.clone(),
        };

        if let Some(identifier) = self.identifier.as_ref() {
            stream_name = format!("{}+{}", stream_name, identifier);
        }

        stream_name
    }

    pub async fn dispatch(&mut self, message_data: MessageData) {
        for handler in self.handlers.iter_mut() {
            let message_data = message_data.clone();
            let executor = self.executor.clone();

            handler.call(message_data, executor);
        }
    }
}

impl<Executor> Consumer<Executor>
where
    Executor: Clone + Send + Sync + 'static,
    for<'e, 'c> &'e Executor: sqlx::Acquire<'c, Database = sqlx::Postgres>,
    for<'e, 'c> &'e Executor: sqlx::PgExecutor<'c>,
{
    pub async fn start(mut self) {
        let mut get = GetCategoryMessages::new(self.executor.clone(), &self.category);

        if let Some(correlation) = self.correlation.as_ref() {
            get.correlation(correlation);
        }

        if let Some(position) = self.get_position().await {
            get.position(position);
        }

        get.batch_size(self.batch_size);

        let poll_interval_millis = self.poll_interval_millis;
        let (dispatch_channel, mut dispatch_receiver) =
            channel::<MessageData>(self.batch_size as usize);

        tokio::task::spawn(async move {
            Subscribtion::new(get, poll_interval_millis, dispatch_channel)
                .start()
                .await
        });

        loop {
            if let Some(message_data) = dispatch_receiver.recv().await {
                let update_position = message_data.global_position;

                self.dispatch(message_data).await;
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
            write.execute(&stream_name).await.unwrap();

            self.position_update_counter = 0;
        }
    }

    pub async fn get_position(&mut self) -> Option<i64> {
        let message_data = GetLastStreamMessage::new(self.executor.clone())
            .execute(&self.position_stream_name())
            .await
            .ok()??;

        let recorded = Msg::<Recorded>::from_data(message_data).ok()?;

        Some(recorded.position)
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

impl<Executor: Clone> Subscribtion<Executor>
where
    for<'e, 'c> &'e Executor: sqlx::PgExecutor<'c>,
{
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Recorded {
    position: i64,
}

impl Message for Recorded {
    const TYPE_NAME: &'static str = "Recorded";
}
