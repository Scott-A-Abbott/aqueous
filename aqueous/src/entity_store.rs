use crate::{
    message_store::{
        error::Error as MessageStoreError, get::GetVersion, Connection, Read, Version,
    },
    stream_name::{Category, StreamID, StreamName},
    Message, MessageData, Msg,
};
use moka::future::Cache;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, OnceLock},
};
use thiserror::Error;
use tracing::{debug, instrument, trace};

type EntityCacheRef = Arc<Box<dyn Any + Send + Sync>>;
type EntityCaches = Cache<TypeId, EntityCacheRef>;
static ENTITY_CACHES: OnceLock<EntityCaches> = OnceLock::new();

#[derive(Error, Debug)]
#[error("A projection that recieves {message_type} already exists")]
pub struct DuplicateProjectionError {
    message_type: String,
}
impl DuplicateProjectionError {
    pub fn new(message_type: String) -> Self {
        Self { message_type }
    }
}

pub struct EntityStore<Entity> {
    projections: HashMap<String, Box<dyn Projection<Entity> + Send>>,
    catchall: Option<Box<dyn Projection<Entity> + Send>>,
    cache: Cache<StreamName, (Entity, Version)>,
    category: Category,
    connection: Connection,
}

impl<Entity> EntityStore<Entity>
where
    Entity: Clone + Send + Sync + 'static,
{
    pub fn build(connection: Connection, category: Category) -> Self {
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                let entity_caches = ENTITY_CACHES.get_or_init(|| Cache::new(5));
                let cache_any = entity_caches
                    .get_with(
                        TypeId::of::<Cache<StreamName, (Entity, Version)>>(),
                        async move {
                            let cache: Cache<StreamName, (Entity, Version)> = Cache::new(10_000);
                            let boxed_cache: Box<dyn Any + Send + Sync> = Box::new(cache);

                            Arc::new(boxed_cache)
                        },
                    )
                    .await;

                let cache = cache_any
                    .downcast_ref::<Cache<StreamName, (Entity, Version)>>()
                    .unwrap();

                Self {
                    projections: HashMap::new(),
                    catchall: None,
                    cache: cache.clone(),
                    category,
                    connection,
                }
            })
        })
    }

    pub fn catchall<F>(&mut self, catchall: F) -> &mut Self
    where
        for<'e> F: Fn(&'e mut Entity, MessageData) + Send + 'static,
    {
        let boxed_projection: Box<dyn Projection<Entity> + Send> = Box::new(catchall);
        self.catchall = Some(boxed_projection);

        self
    }

    pub fn insert_projection<F, M>(
        &mut self,
        func: F,
    ) -> Result<&mut Self, DuplicateProjectionError>
    where
        for<'de> M: Message + Send + serde::Deserialize<'de> + 'static,
        F: Fn(&mut Entity, Msg<M>) + Send + 'static,
        Entity: 'static,
    {
        let message_type = M::TYPE_NAME.to_string();

        if self.projections.contains_key(&message_type) {
            return Err(DuplicateProjectionError::new(message_type));
        }

        let projection = IntoProjection::into_projection(func);
        let boxed_projection: Box<dyn Projection<Entity> + Send> = Box::new(projection);

        self.projections.insert(message_type, boxed_projection);

        Ok(self)
    }

    pub fn extend_projections<M>(
        &mut self,
        collection: impl IntoProjectionCollection<Entity, M>,
    ) -> Result<&mut Self, DuplicateProjectionError> {
        let ProjectionCollection { projections, .. } = collection.into_projection_collection();

        for (message_type, projection) in projections.into_iter() {
            if self.projections.contains_key(&message_type) {
                return Err(DuplicateProjectionError::new(message_type));
            }

            self.projections.insert(message_type, projection);
        }

        Ok(self)
    }
}

impl<Entity> EntityStore<Entity>
where
    Entity: Default + Debug + Clone + Send + Sync + 'static,
{
    #[instrument(skip_all, name = "EntityStore::fetch")]
    pub async fn fetch(
        &mut self,
        stream_id: StreamID,
    ) -> Result<(Entity, Version), MessageStoreError> {
        let stream_name = self.category.stream_name(stream_id);
        debug!("Stream name: {}", stream_name);

        let (mut entity, mut version) = self
            .cache
            .get_with(stream_name.clone(), async {
                (Entity::default(), Version::initial())
            })
            .await;

        if version != Version::initial() {
            debug!("Fetched cahced entity at version {}: {:?}", version, entity);
        }

        let current_version = GetVersion::new(self.connection.clone())
            .execute(stream_name.clone())
            .await?;

        if version == current_version {
            debug!("Stream version matches cached entity version");
            return Ok((entity, version));
        }

        let messages = Read::build(self.connection.clone())
            .position(version.0 + 1)
            .execute(stream_name.clone())
            .await?;

        trace!("Messages: {:?}", messages);

        for message in messages {
            let position = message.metadata.position().unwrap();
            version.0 = position;

            for projection in self.projections.values() {
                projection.apply(&mut entity, message.clone());
            }

            if let Some(catchall) = self.catchall.as_ref() {
                catchall.apply(&mut entity, message);
            }
        }

        self.cache
            .insert(stream_name, (entity.clone(), version.clone()))
            .await;

        debug!("Newly cached entity at version {}: {:?}", version, entity);

        Ok((entity, version))
    }
}

pub trait Projection<Entity> {
    fn apply(&self, entity: &mut Entity, message_data: MessageData);
}

impl<'e, Entity, F> Projection<Entity> for F
where
    F: Fn(&mut Entity, MessageData),
{
    fn apply(&self, entity: &mut Entity, message_data: MessageData) {
        self(entity, message_data);
    }
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
        if &message_data.type_name == M::TYPE_NAME {
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
    projections: HashMap<String, Box<dyn Projection<Entity> + Send>>,
    marker: PhantomData<Marker>,
}

pub trait IntoProjectionCollection<Entity, Marker> {
    fn into_projection_collection(self) -> ProjectionCollection<Entity, Marker>;
}

impl<Entity, M, F> IntoProjectionCollection<Entity, (&'static mut Entity, Msg<M>)> for F
where
    Entity: Send,
    F: Fn(&mut Entity, Msg<M>)
        + IntoProjection<(&'static mut Entity, Msg<M>)>
        + Send
        + Sized
        + 'static,
    for<'de> M: Message + serde::Deserialize<'de> + Send + 'static,
{
    fn into_projection_collection(
        self,
    ) -> ProjectionCollection<Entity, (&'static mut Entity, Msg<M>)> {
        let projection = self.into_projection();
        let message_type = M::TYPE_NAME.to_string();
        let boxed_projection: Box<dyn Projection<Entity> + Send> = Box::new(projection);

        let mut projections = HashMap::new();
        projections.insert(message_type, boxed_projection);

        ProjectionCollection {
            projections,
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
                let mut projections = HashMap::new();

                $( let ProjectionCollection { projections: $fn, .. } = $fn.into_projection_collection(); )*
                $( projections.extend($fn.into_iter()); )*

                ProjectionCollection {
                    projections,
                    marker: Default::default(),
                }
            }
        }
    }
}

all_projection_tuples!(impl_into_projection_collection);
