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
    cache: Cache<StreamName, (Entity, Version)>,
    category: Category,
    pool: PgPool,
}

impl<Entity> EntityStore<Entity>
where
    Entity: Clone + Send + Sync + 'static,
{
    pub fn build(pool: PgPool, category: Category) -> Self {
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                let entity_cache = ENTITY_CACHE.get_or_init(|| Cache::new(5));
                let cache_any = entity_cache
                    .get_with(
                        TypeId::of::<Cache<StreamName, (Entity, Version)>>(),
                        async move {
                            let cache: Cache<String, (Entity, Version)> = Cache::new(10_000);
                            let boxed_cache: Box<dyn Any + Send + Sync> = Box::new(cache);

                            Arc::new(boxed_cache)
                        },
                    )
                    .await;

                let cache = cache_any
                    .downcast_ref::<Cache<StreamName, (Entity, Version)>>()
                    .unwrap();

                Self {
                    projections: Vec::new(),
                    cache: cache.clone(),
                    category,
                    pool,
                }
            })
        })
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

    pub fn add_projections<M>(&mut self, collection: impl IntoProjectionCollection<Entity, M>) -> &mut Self {
        let ProjectionCollection { mut projections, .. } = collection.into_projection_collection();
        self.projections.append(&mut projections);

        self
    }
}

impl<Entity> EntityStore<Entity>
where
    Entity: Default + Clone + Send + Sync + 'static,
{
    pub async fn fetch(&mut self, stream_id: StreamID) -> (Entity, Version) {
        let stream_name = self.category.stream_name(stream_id);

        let (mut entity, mut version) = self
            .cache
            .get_with(stream_name.clone(), async {
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
            for projection in self.projections.iter() {
                projection.apply(&mut entity, message.clone());
            }

            version.0 = message.position;
        }

        self.cache
            .insert(stream_name, (entity.clone(), version.clone()))
            .await;

        (entity, version)
    }
}

pub trait Projection<Entity> {
    fn apply(&self, entity: &mut Entity, message_data: MessageData);
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
    fn apply(&self, entity: &mut Entity, message_data: MessageData) {
        if message_data.type_name == M::TYPE_NAME.to_string() {
            let msg = Msg::<M>::from_data(message_data).unwrap();
            (self.func)(entity, msg);
        }
    }
}

pub trait IntoProjection<Marker>: Sized {
    fn into_projection(self) -> FunctionProjection<Marker, Self>;
}

impl<'e, Entity, M, F> IntoProjection<(&'e mut Entity, Msg<M>)> for F
where
    for<'de> M: Message + serde::Deserialize<'de>,
    F: Fn(&mut Entity, Msg<M>),
{
    fn into_projection(self) -> FunctionProjection<(&'e mut Entity, Msg<M>), Self> {
        FunctionProjection {
            func: self,
            marker: Default::default(),
        }
    }
}

pub struct ProjectionCollection<Entity, Marker> {
    projections: Vec<Box<dyn Projection<Entity> + Send>>,
    marker: PhantomData<Marker>,
}

pub trait IntoProjectionCollection<Entity, Marker> {
    fn into_projection_collection(self) -> ProjectionCollection<Entity, Marker>;
}

impl<Entity, M, F> IntoProjectionCollection<Entity, (&'static mut Entity, Msg<M>)> for F
where
    Entity: Send,
    F: Fn(&mut Entity, Msg<M>) + IntoProjection<(&'static mut Entity, Msg<M>)> + Send + Sized + 'static,
    for<'de> M: Message + serde::Deserialize<'de> + Send + 'static,
{
    fn into_projection_collection(self) -> ProjectionCollection<Entity, (&'static mut Entity, Msg<M>)> {
        let projection = self.into_projection();
        let boxed_projection: Box<dyn Projection<Entity> + Send> = Box::new(projection);

        ProjectionCollection {
            projections: vec![boxed_projection],
            marker: Default::default(),
        }
    }
}

#[rustfmt::skip]
macro_rules! all_projection_tuples {
    ($macro_name:ident) => {
        $macro_name!((M1, F1));
        $macro_name!((M1, F1), (M2, F2));
        $macro_name!((M1, F1), (M2, F2), (M3, F3));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7), (M8, F8));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7), (M8, F8), (M9, F9));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7), (M8, F8), (M9, F9), (M10, F10));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7), (M8, F8), (M9, F9), (M10, F10), (M11, F11));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7), (M8, F8), (M9, F9), (M10, F10), (M11, F11), (M12, F12));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7), (M8, F8), (M9, F9), (M10, F10), (M11, F11), (M12, F12), (M13, F13));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7), (M8, F8), (M9, F9), (M10, F10), (M11, F11), (M12, F12), (M13, F13), (M14, F14));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7), (M8, F8), (M9, F9), (M10, F10), (M11, F11), (M12, F12), (M13, F13), (M14, F14), (M15, F15));
        $macro_name!((M1, F1), (M2, F2), (M3, F3), (M4, F4), (M5, F5), (M6, F6), (M7, F7), (M8, F8), (M9, F9), (M10, F10), (M11, F11), (M12, F12), (M13, F13), (M14, F14), (M15, F15), (M16, F16));
    };
}

macro_rules! impl_into_projection_collection {
    ($(($msg:ident, $fn:ident) $(,)?)*) => {
        #[allow(non_snake_case)]
        impl<Entity, $($msg, $fn),*> IntoProjectionCollection<Entity, ($((&'static mut Entity, Msg<$msg>),)*)> for ($($fn,)*)
        where
            $(for<'de> $msg: Message + serde::Deserialize<'de> + 'static,)*
            $($fn: IntoProjectionCollection<Entity, (&'static mut Entity, Msg<$msg>)>),*
        {
            fn into_projection_collection(self) -> ProjectionCollection<Entity, ($((&'static mut Entity, Msg<$msg>),)*)> {
                let ($($fn,)*) = self;
                let mut projections = Vec::new();

                $( let ProjectionCollection { projections: mut $fn, .. } = $fn.into_projection_collection(); )*
                $( projections.append(&mut $fn); )*

                ProjectionCollection {
                    projections,
                    marker: Default::default(),
                }
            }
        }
    }
}

all_projection_tuples!(impl_into_projection_collection);
