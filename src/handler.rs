use std::{
    any::{Any, TypeId},
    cell::{RefCell, UnsafeCell},
    collections::HashMap,
    error::Error,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub trait Handler {
    fn call(&mut self, message_data: crate::MessageData);
    fn handles_message(&self, message_type: &str) -> bool;
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

pub trait IntoHandler<ParamsMarker, ReturnMarker>: Sized {
    fn into_handler(this: Self) -> FunctionHandler<ParamsMarker, ReturnMarker, Self>;
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

macro_rules! impl_handler {
   ([$($ty:ident $(,)?)*], $re:ident)=> {
        #[allow(non_snake_case, unused_mut)]
        impl<$($ty,)* $re, F> Handler for FunctionHandler<($($ty,)*), $re, F>
        where
            $($ty: HandlerParam,)*
            F: FnMut($($ty,)*) -> $re,
            $re: std::future::Future<Output = ()>,
        {
            fn call(&mut self, message_data: crate::MessageData) {
                $(let $ty = $ty::build(message_data.clone(), &self.resources);)*
                tokio::task::block_in_place(move || {
                    tokio::runtime::Handle::current().block_on(async move {
                        (self.func)($($ty,)*).await;
                    });
                });
            }

            fn handles_message(&self, message_type: &str) -> bool {
                message_type == self.message_type.as_str()
            }
        }
   }
}

all_tuples!(impl_handler);

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

macro_rules! impl_into_handler {
    ($first:ident, [$($ty:ident $(,)?)*], $re:ident) => {
        #[allow(non_snake_case, unused_mut)]
        impl<Msg, F, $first, $($ty,)* $re> IntoHandler<($first, $($ty,)*), $re> for F
        where
            Msg: crate::Message,
            $first: Deref<Target = Msg>,
            F: FnMut($first, $($ty,)*) -> $re,
        {
            fn into_handler(this: Self) -> FunctionHandler<($first, $($ty,)*), $re, Self> {
                FunctionHandler {
                    func: this,
                    params_marker: Default::default(),
                    return_marker: Default::default(),
                    message_type: Msg::TYPE_NAME.to_owned(),
                    resources: Default::default(),
                }
            }
        }
    }
}

all_tuples_with_first!(impl_into_handler);
