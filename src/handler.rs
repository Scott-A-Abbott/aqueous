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

    fn build(
        message_data: crate::MessageData,
        retainers: &mut HandlerRetainers,
    ) -> Result<Self, Self::Error>;
}

#[derive(Default)]
pub struct HandlerRetainers {
    inner: HashMap<TypeId, Box<dyn Any>>,
}
impl HandlerRetainers {
    pub fn insert<T: 'static>(&mut self, retain: T) {
        let type_id = TypeId::of::<T>();
        let retainer = Retain::new(retain);
        let boxed_retainer: Box<dyn Any> = Box::new(retainer);
        self.inner.insert(type_id, boxed_retainer);
    }

    pub fn get<T: 'static>(&mut self) -> Option<Retain<T>> {
        let type_id = TypeId::of::<T>();
        let boxed_any = self.inner.remove(&type_id)?;
        let retainer: Retain<T> = *boxed_any.downcast().ok()?;

        self.inner.insert(type_id, Box::new(retainer.clone()));
        Some(retainer)
    }

    pub fn contains<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.inner.contains_key(&type_id)
    }
}

pub struct Retain<T>(Rc<UnsafeCell<T>>);
impl<T> Retain<T> {
    pub fn new(retain: T) -> Retain<T> {
        let cell = UnsafeCell::new(retain);
        let rc = Rc::new(cell);
        Self(rc)
    }
}
impl<T> Clone for Retain<T> {
    fn clone(&self) -> Self {
        let rc = self.0.clone();
        Self(rc)
    }
}
impl<T> Deref for Retain<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}
impl<T> DerefMut for Retain<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.get() }
    }
}
impl<T: HandlerParam + 'static> HandlerParam for Retain<T> {
    type Error = Box<dyn Error>;

    fn build(
        message_data: crate::MessageData,
        retainers: &mut HandlerRetainers,
    ) -> Result<Self, Self::Error> {
        if retainers.contains::<T>() {
            let retainer = retainers.get::<T>().unwrap();
            Ok(retainer)
        } else {
            let retainer = T::build(message_data, retainers).ok().unwrap();

            retainers.insert(retainer);
            let retainer = retainers.get::<T>().unwrap();

            Ok(retainer)
        }
    }
}

pub struct FunctionHandler<Marker, F> {
    func: F,
    marker: PhantomData<Marker>,
    message_type: String,
    retainers: HandlerRetainers,
}

impl<T1, F> Handler for FunctionHandler<(T1,), F>
where
    T1: HandlerParam,
    F: FnMut(T1),
{
    fn call(&mut self, message_data: crate::MessageData) {
        let t1 = T1::build(message_data, &mut self.retainers).unwrap();
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
        let t1 = T1::build(message_data.clone(), &mut self.retainers).unwrap();
        let t2 = T2::build(message_data, &mut self.retainers).unwrap();
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
            retainers: Default::default(),
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
            retainers: Default::default(),
        }
    }
}
// #[rustfmt::skip]
// macro_rules! for_all_tuples {
//     ($maro_name:ident) => {
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
