use std::{collections::HashMap, marker::PhantomData};

pub struct Version(pub i64);
impl Version {
    pub fn initial() -> Self {
        Self(-1)
    }
}

pub struct Store<Entity, Executor = sqlx::PgPool> {
    projections: Vec<Box<dyn Projection<Entity>>>,
    entries: HashMap<String, (i64, Entity)>,
    executor: Executor,
}
// ## FIX THIS FOR REALZIES!!!!
unsafe impl<Entity> Send for Store<Entity> {}

impl<Entity, Executor> Store<Entity, Executor> {
    pub fn new(executor: Executor) -> Self {
        Self {
            projections: Vec::new(),
            entries: HashMap::new(),
            executor,
        }
    }

    pub fn with_projection<F, Message>(&mut self, func: F) -> &mut Self
    where
        for<'de> Message: crate::Message + serde::Deserialize<'de> + 'static,
        F: FnMut(&mut Entity, crate::Msg<Message>) + 'static,
        Entity: 'static,
    {
        let projection = IntoProjection::into_projection(func);
        let boxed_projection: Box<dyn Projection<Entity>> = Box::new(projection);

        self.projections.push(boxed_projection);

        self
    }
}

impl<Entity: Default> Store<Entity> {
    pub async fn fetch(&mut self, stream_name: &str) -> (&Entity, Version) {
        let (pos, entity) = self.entries.entry(String::from(stream_name)).or_default();
        let messages = crate::GetStreamMessages::new(self.executor.clone(), stream_name)
            .position(*pos)
            .execute()
            .await
            .unwrap();

        for message in messages {
            for projection in self.projections.iter_mut() {
                if projection.applies_message(&message.type_name) {
                    projection.apply(entity, message.clone());
                }
            }

            *pos = message.position;
        }

        (entity, Version(*pos))
    }
}

impl<Entity, Executor> crate::HandlerParam for Store<Entity, Executor>
where
    Executor: Clone + 'static,
{
    type Error = Box<dyn std::error::Error>;

    fn build(
        _: crate::MessageData,
        resources: &crate::HandlerResources,
    ) -> Result<Self, Self::Error> {
        use std::ops::Deref;

        let executor_resource: crate::Res<Executor> = resources.get().unwrap();
        let executor = executor_resource.deref();
        let store = Store::new(executor.clone());
        Ok(store)
    }
}

pub trait Projection<Entity> {
    fn apply(&mut self, entity: &mut Entity, message_data: crate::MessageData);
    fn applies_message(&self, message_type: &str) -> bool;
}

pub struct FunctionProjection<Marker, F> {
    func: F,
    message_type: String,
    marker: PhantomData<Marker>,
}

impl<'e, Entity, Message, F> Projection<Entity>
    for FunctionProjection<(&'e mut Entity, crate::Msg<Message>), F>
where
    for<'de> Message: crate::Message + serde::Deserialize<'de>,
    F: FnMut(&mut Entity, crate::Msg<Message>),
{
    fn apply(&mut self, entity: &mut Entity, message_data: crate::MessageData) {
        let msg = crate::Msg::<Message>::from_data(message_data).unwrap();
        (self.func)(entity, msg);
    }

    fn applies_message(&self, message_type: &str) -> bool {
        message_type == self.message_type
    }
}

pub trait IntoProjection<Marker>: Sized {
    fn into_projection(this: Self) -> FunctionProjection<Marker, Self>;
}

impl<'e, Entity, Message, F> IntoProjection<(&'e mut Entity, crate::Msg<Message>)> for F
where
    for<'de> Message: crate::Message + serde::Deserialize<'de>,
    F: FnMut(&mut Entity, crate::Msg<Message>),
{
    fn into_projection(
        this: Self,
    ) -> FunctionProjection<(&'e mut Entity, crate::Msg<Message>), Self> {
        FunctionProjection {
            func: this,
            message_type: Message::TYPE_NAME.to_owned(),
            marker: Default::default(),
        }
    }
}
