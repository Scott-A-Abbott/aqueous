use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
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
    type Error: std::fmt::Debug;

    fn initialize(
        message_data: crate::MessageData,
        dependencies: &mut HandlerDependencies,
    ) -> Result<Self, Self::Error>;
}

#[derive(Default)]
pub struct HandlerDependencies {
    inner: HashMap<TypeId, Box<dyn Any>>,
}
impl HandlerDependencies {
    pub fn insert<T: 'static>(&mut self, dependency: T) {
        let type_id = TypeId::of::<T>();
        let dep = Dep::new(dependency);
        let boxed_dep: Box<dyn Any> = Box::new(dep);
        self.inner.insert(type_id, boxed_dep);
    }

    pub fn get<T: 'static>(&mut self) -> Option<Dep<T>> {
        let type_id = TypeId::of::<T>();
        let boxed_any = self.inner.remove(&type_id)?;
        let dependency: Dep<T> = *boxed_any.downcast().ok()?;

        self.inner.insert(type_id, Box::new(dependency.clone()));
        Some(dependency)
    }

    pub fn contains<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.inner.contains_key(&type_id)
    }
}

pub struct Dep<T>(Rc<UnsafeCell<T>>);
impl<T> Dep<T> {
    pub fn new(dependency: T) -> Dep<T> {
        let cell = UnsafeCell::new(dependency);
        let rc = Rc::new(cell);
        Dep(rc)
    }
}
impl<T> Clone for Dep<T> {
    fn clone(&self) -> Self {
        let rc = self.0.clone();
        Dep(rc)
    }
}
impl<T> Deref for Dep<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}
impl<T> DerefMut for Dep<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.get() }
    }
}
impl<T: HandlerParam + 'static> HandlerParam for Dep<T> {
    type Error = Box<dyn Error>;

    fn initialize(
        message_data: crate::MessageData,
        dependencies: &mut HandlerDependencies,
    ) -> Result<Self, Self::Error> {
        if dependencies.contains::<T>() {
            let dep = dependencies.get::<T>().unwrap();
            Ok(dep)
        } else {
            let dependency = T::initialize(message_data, dependencies).ok().unwrap();

            dependencies.insert(dependency);
            let dep = dependencies.get::<T>().unwrap();

            Ok(dep)
        }
    }
}

pub struct FunctionHandler<Marker, F> {
    func: F,
    marker: PhantomData<Marker>,
    message_type: String,
    dependencies: HandlerDependencies,
}

impl<T1, F> Handler for FunctionHandler<(T1,), F>
where
    T1: HandlerParam,
    F: FnMut(T1),
{
    fn call(&mut self, message_data: crate::MessageData) {
        let t1 = T1::initialize(message_data, &mut self.dependencies).unwrap();
        (self.func)(t1);
    }

    fn handles_message(&self, message_type: &str) -> bool {
        message_type == self.message_type.as_str()
    }
}

impl<T1, T2, F> Handler for FunctionHandler<(T1, T2), F>
where
    T1: HandlerParam,
    T2: HandlerParam,
    F: FnMut(T1, T2),
{
    fn call(&mut self, message_data: crate::MessageData) {
        let t1 = T1::initialize(message_data.clone(), &mut self.dependencies).unwrap();
        let t2 = T2::initialize(message_data, &mut self.dependencies).unwrap();
        (self.func)(t1, t2);
    }

    fn handles_message(&self, message_type: &str) -> bool {
        message_type == self.message_type.as_str()
    }
}

pub trait IntoHandler<Marker>: Sized {
    fn into_handler(this: Self) -> FunctionHandler<Marker, Self>;
}

impl<Msg, T1, F> IntoHandler<(T1,)> for F
where
    Msg: crate::Message,
    T1: Deref<Target = Msg>,
    F: FnMut(T1),
{
    fn into_handler(this: Self) -> FunctionHandler<(T1,), Self> {
        FunctionHandler {
            func: this,
            marker: Default::default(),
            message_type: Msg::TYPE_NAME.to_owned(),
            dependencies: Default::default(),
        }
    }
}

impl<Msg, T1, T2, F> IntoHandler<(T1, T2)> for F
where
    Msg: crate::Message,
    T1: Deref<Target = Msg>,
    F: FnMut(T1, T2),
{
    fn into_handler(this: Self) -> FunctionHandler<(T1, T2), Self> {
        FunctionHandler {
            func: this,
            marker: Default::default(),
            message_type: Msg::TYPE_NAME.to_owned(),
            dependencies: Default::default(),
        }
    }
}
// #[rustfmt::skip]
// macro_rules! for_all_tuples {
//     ($macro_name:ident) => {
//         $macro_name!(T1);
//         $macro_name!(T1, T2);
//         $macro_name!(T1, T2, T3);
//         $macro_name!(T1, T2, T3, T4);
//         $macro_name!(T1, T2, T3, T4, T5);
//         $macro_name!(T1, T2, T3, T4, T5, T6);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
//     };
// }
