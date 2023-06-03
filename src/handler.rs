use std::{
    any::{Any, TypeId},
    cell::{RefCell, UnsafeCell},
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub trait Handler {
    fn call(&mut self, message_data: crate::MessageData);
}

pub trait HandlerParam: Sized {
    fn build(message_data: crate::MessageData, resources: &HandlerResources) -> Self;
}

#[derive(Default)]
pub struct HandlerResources(RefCell<HashMap<TypeId, Box<dyn Any>>>);
impl HandlerResources {
    pub fn insert<T: 'static>(&self, resource: T) {
        let res = Res::new(resource);
        let boxed_res: Box<dyn Any> = Box::new(res);
        let type_id = TypeId::of::<T>();

        let mut resources = self.0.take();
        resources.insert(type_id, boxed_res);

        self.0.replace(resources);
    }

    pub fn get<T: 'static>(&self) -> Option<Res<T>> {
        let mut resources = self.0.take();

        let type_id = TypeId::of::<T>();
        let boxed_any = resources.remove(&type_id)?;
        let resource: Res<T> = *boxed_any.downcast().ok()?;

        resources.insert(type_id, Box::new(resource.clone()));
        self.0.replace(resources);

        Some(resource)
    }

    pub fn contains<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.0.borrow().contains_key(&type_id)
    }
}

pub struct Res<T>(Rc<UnsafeCell<T>>);
impl<T> Res<T> {
    pub fn new(resource: T) -> Res<T> {
        let cell = UnsafeCell::new(resource);
        let rc = Rc::new(cell);
        Self(rc)
    }
}
impl<T> Clone for Res<T> {
    fn clone(&self) -> Self {
        let rc = self.0.clone();
        Self(rc)
    }
}
impl<T> Deref for Res<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}
impl<T> DerefMut for Res<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.get() }
    }
}
impl<T: HandlerParam + 'static> HandlerParam for Res<T> {
    fn build(message_data: crate::MessageData, resources: &HandlerResources) -> Self {
        if resources.contains::<T>() {
            let resource = resources.get::<T>().unwrap();
            resource
        } else {
            let resource = T::build(message_data, resources);

            resources.insert(resource);
            let resource = resources.get::<T>().unwrap();

            resource
        }
    }
}

pub struct FunctionHandler<ParamsMarker, ReturnMarker, F> {
    func: F,
    params_marker: PhantomData<ParamsMarker>,
    return_marker: PhantomData<ReturnMarker>,
    pub message_type: String,
    pub resources: HandlerResources,
}

pub trait IntoHandler<ParamsMarker, ReturnMarker, Func>: Sized {
    fn into_handler(self) -> FunctionHandler<ParamsMarker, ReturnMarker, Func>;
    fn insert_resource<R: 'static>(
        self,
        resource: R,
    ) -> FunctionHandler<ParamsMarker, ReturnMarker, Func>;
}

#[rustfmt::skip]
macro_rules! all_tuples {
    ($macro_name:ident) => {
        $macro_name!([T1], R);
        $macro_name!([T1, T2], R);
        $macro_name!([T1, T2, T3], R);
        $macro_name!([T1, T2, T3, T4], R);
        $macro_name!([T1, T2, T3, T4, T5], R);
        $macro_name!([T1, T2, T3, T4, T5, T6], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7, T8], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], R);
        $macro_name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16], R);
    };
}

macro_rules! impl_into_handler_self {
    ([$($ty:ident $(,)?)*], $re:ident)=> {
        #[allow(non_snake_case, unused_mut)]
        impl<Func, $($ty,)* $re> IntoHandler<($($ty,)*), $re, Func> for FunctionHandler<($($ty,)*), $re, Func>
        where
            Func: FnMut($($ty,)*) -> $re,
        {
            fn into_handler(self) -> Self {
                self
            }

            fn insert_resource<Res: 'static>(self, resource: Res) -> Self {
                self.resources.insert(resource);
                self
            }
        }
    }
}

all_tuples!(impl_into_handler_self);

#[rustfmt::skip]
macro_rules! all_tuples_with_first {
    ($macro_name:ident) => {
        $macro_name!(T1, [], R);
        $macro_name!(T1, [T2], R);
        $macro_name!(T1, [T2, T3], R);
        $macro_name!(T1, [T2, T3, T4], R);
        $macro_name!(T1, [T2, T3, T4, T5], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], R);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16], R);
    };
}

macro_rules! impl_handler {
   ($first:ident, [$($ty:ident $(,)?)*], $re:ident)=> {
        #[allow(non_snake_case, unused_mut)]
        impl<$first, $($ty,)* $re, F> Handler for FunctionHandler<(crate::Msg<$first>, $($ty,)*), $re, F>
        where
            for<'de> $first: crate::Message + serde::Deserialize<'de>,
            $($ty: HandlerParam,)*
            F: FnMut(crate::Msg<$first>, $($ty,)*) -> $re,
            $re: std::future::Future<Output = ()>,
        {
            fn call(&mut self, message_data: crate::MessageData) {
                if message_data.type_name == $first::TYPE_NAME.to_string() {
                    let msg: crate::Msg<$first> = crate::Msg::build(message_data.clone(), &self.resources);
                    $(let $ty = $ty::build(message_data.clone(), &self.resources);)*
                    tokio::task::block_in_place(move || {
                        tokio::runtime::Handle::current().block_on(async move {
                            (self.func)(msg, $($ty,)*).await;
                        });
                    });
                }
            }
        }
    }
}

macro_rules! impl_into_handler {
    ($first:ident, [$($ty:ident $(,)?)*], $re:ident) => {
        #[allow(non_snake_case, unused_mut)]
        impl<Msg, Func, $first, $($ty,)* $re> IntoHandler<($first, $($ty,)*), $re, Func> for Func
        where
            Msg: crate::Message,
            $first: Deref<Target = Msg>,
            Func: FnMut($first, $($ty,)*) -> $re,
        {
            fn into_handler(self) -> FunctionHandler<($first, $($ty,)*), $re, Self> {
                FunctionHandler {
                    func: self,
                    params_marker: Default::default(),
                    return_marker: Default::default(),
                    message_type: Msg::TYPE_NAME.to_owned(),
                    resources: Default::default(),
                }
            }

            fn insert_resource<Res: 'static>(self, resource: Res) -> FunctionHandler<($first, $($ty,)*), $re, Self> {
                let handler = self.into_handler();
                handler.resources.insert(resource);

                handler
            }
        }
    }
}

all_tuples_with_first!(impl_handler);
all_tuples_with_first!(impl_into_handler);
