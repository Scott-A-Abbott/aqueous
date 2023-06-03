use moka::future::Cache;
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

static ENTITY_CACHE: OnceLock<Cache<TypeId, Arc<Box<dyn Any + Send + Sync>>>> = OnceLock::new();

#[derive(Copy, Clone)]
pub struct Version(pub i64);
impl Version {
    pub fn initial() -> Self {
        Self(-1)
    }
}
impl Default for Version {
    fn default() -> Self {
        Version::initial()
    }
}

pub struct Store<Entity, Executor = sqlx::PgPool> {
    projections: Vec<Box<dyn Projection<Entity>>>,
    cache: Cache<String, (Entity, Version)>,
    executor: Executor,
}

impl<Entity, Executor> Store<Entity, Executor>
where
    Entity: Clone + Send + Sync + 'static,
{
    pub fn new(executor: Executor, cache: Cache<String, (Entity, Version)>) -> Self {
        Self {
            projections: Vec::new(),
            cache,
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

impl<Entity> Store<Entity>
where
    Entity: Default + Clone + Send + Sync + 'static,
{
    pub async fn fetch(&mut self, stream_name: &str) -> (Entity, Version) {
        let (mut entity, mut version) = self
            .cache
            .get_with(stream_name.to_string(), async {
                (Entity::default(), Version::default())
            })
            .await;

        let messages = crate::GetStreamMessages::new(self.executor.clone(), stream_name)
            .position(version.0)
            .execute()
            .await
            .unwrap();

        for message in messages {
            for projection in self.projections.iter_mut() {
                projection.apply(&mut entity, message.clone());
            }

            version.0 = message.position;
        }

        self.cache
            .insert(stream_name.to_string(), (entity.clone(), version.clone()))
            .await;

        (entity, version)
    }
}

impl<Entity, Executor> crate::HandlerParam<Executor> for Store<Entity, Executor>
where
    Entity: Clone + Send + Sync + 'static,
    Executor: Clone + 'static,
{
    fn build(_: crate::MessageData, executor: Executor) -> Self {
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                let entity_cache = ENTITY_CACHE.get_or_init(|| Cache::new(5));
                let cache_any = entity_cache
                    .get_with(
                        TypeId::of::<Cache<String, (Entity, Version)>>(),
                        async move {
                            let cache: Cache<String, (Entity, Version)> = Cache::new(10_000);
                            let boxed_cache: Box<dyn Any + Send + Sync> = Box::new(cache);

                            Arc::new(boxed_cache)
                        },
                    )
                    .await;

                let cache = cache_any
                    .downcast_ref::<Cache<String, (Entity, Version)>>()
                    .unwrap();

                let store = Store::new(executor.clone(), cache.clone());
                store
            })
        })
    }
}

pub trait Projection<Entity> {
    fn apply(&mut self, entity: &mut Entity, message_data: crate::MessageData);
}

pub struct FunctionProjection<Marker, F> {
    func: F,
    marker: PhantomData<Marker>,
}

impl<'e, Entity, Message, F> Projection<Entity>
    for FunctionProjection<(&'e mut Entity, crate::Msg<Message>), F>
where
    for<'de> Message: crate::Message + serde::Deserialize<'de>,
    F: FnMut(&mut Entity, crate::Msg<Message>),
{
    fn apply(&mut self, entity: &mut Entity, message_data: crate::MessageData) {
        if message_data.type_name == Message::TYPE_NAME.to_string() {
            let msg = crate::Msg::<Message>::from_data(message_data).unwrap();
            (self.func)(entity, msg);
        }
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
            marker: Default::default(),
        }
    }
}
