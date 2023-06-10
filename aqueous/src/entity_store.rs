use crate::*;
use moka::future::Cache;
use sqlx::PgPool;
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

static ENTITY_CACHE: OnceLock<Cache<TypeId, Arc<Box<dyn Any + Send + Sync>>>> = OnceLock::new();

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Version(pub i64);
impl Version {
    pub fn initial() -> Self {
        Self(-1)
    }
}

pub struct EntityStore<Entity> {
    projections: Vec<Box<dyn Projection<Entity> + Send>>,
    cache: Cache<String, (Entity, Version)>,
    pool: PgPool,
}

impl<Entity> EntityStore<Entity>
where
    Entity: Clone + Send + Sync + 'static,
{
    pub fn new(pool: PgPool, cache: Cache<String, (Entity, Version)>) -> Self {
        Self {
            projections: Vec::new(),
            cache,
            pool,
        }
    }

    pub fn add_projection<F, M>(&mut self, func: F) -> &mut Self
    where
        for<'de> M: Message + Send + serde::Deserialize<'de> + 'static,
        F: Fn(&mut Entity, Msg<M>) + Send + 'static,
        Entity: 'static,
    {
        let projection = IntoProjection::into_projection(func);
        let boxed_projection: Box<dyn Projection<Entity> + Send> = Box::new(projection);

        self.projections.push(boxed_projection);

        self
    }
}

impl<Entity> EntityStore<Entity>
where
    Entity: Default + Clone + Send + Sync + 'static,
{
    pub async fn fetch(&mut self, stream_name: StreamName) -> (Entity, Version) {
        let (mut entity, mut version) = self
            .cache
            .get_with(stream_name.0.clone(), async {
                (Entity::default(), Version::initial())
            })
            .await;

        let current_version = GetStreamVersion::new(self.pool.clone())
            .execute(stream_name.clone())
            .await
            .unwrap();

        if version == current_version {
            return (entity, version);
        }

        let messages = GetStreamMessages::new(self.pool.clone())
            .position(version.0 + 1)
            .execute(stream_name.clone())
            .await
            .unwrap();

        for message in messages {
            for projection in self.projections.iter_mut() {
                projection.apply(&mut entity, message.clone());
            }

            version.0 = message.position;
        }

        self.cache
            .insert(stream_name.0.clone(), (entity.clone(), version.clone()))
            .await;

        (entity, version)
    }
}

impl<Entity, Settings> HandlerParam<Settings> for EntityStore<Entity>
where
    Entity: Clone + Send + Sync + 'static,
{
    fn build(pool: PgPool, _: Settings) -> Self {
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

                let store = Self::new(pool, cache.clone());
                store
            })
        })
    }
}

pub trait Projection<Entity> {
    fn apply(&mut self, entity: &mut Entity, message_data: MessageData);
}

pub struct FunctionProjection<Marker, F> {
    func: F,
    marker: PhantomData<Marker>,
}

impl<'e, Entity, M, F> Projection<Entity> for FunctionProjection<(&'e mut Entity, Msg<M>), F>
where
    for<'de> M: Message + serde::Deserialize<'de>,
    F: Fn(&mut Entity, Msg<M>),
{
    fn apply(&mut self, entity: &mut Entity, message_data: MessageData) {
        if message_data.type_name == M::TYPE_NAME.to_string() {
            let msg = Msg::<M>::from_data(message_data).unwrap();
            (self.func)(entity, msg);
        }
    }
}

pub trait IntoProjection<Marker>: Sized {
    fn into_projection(this: Self) -> FunctionProjection<Marker, Self>;
}

impl<'e, Entity, M, F> IntoProjection<(&'e mut Entity, Msg<M>)> for F
where
    for<'de> M: Message + serde::Deserialize<'de>,
    F: Fn(&mut Entity, Msg<M>),
{
    fn into_projection(this: Self) -> FunctionProjection<(&'e mut Entity, Msg<M>), Self> {
        FunctionProjection {
            func: this,
            marker: Default::default(),
        }
    }
}
