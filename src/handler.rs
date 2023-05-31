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
    type Error: std::fmt::Debug;

    fn build(
        message_data: crate::MessageData,
        retainers: &HandlerRetainers,
    ) -> Result<Self, Self::Error>;
}

#[derive(Default)]
pub struct HandlerRetainers(RefCell<HashMap<TypeId, Box<dyn Any>>>);
impl HandlerRetainers {
    pub fn retain<T: 'static>(&self, retain: T) {
        let retainer = Retain::new(retain);
        let boxed_retainer: Box<dyn Any> = Box::new(retainer);
        let type_id = TypeId::of::<T>();

        let mut retainers = self.0.take();
        retainers.insert(type_id, boxed_retainer);

        self.0.replace(retainers);
    }

    pub fn get<T: 'static>(&self) -> Option<Retain<T>> {
        let mut retainers = self.0.take();

        let type_id = TypeId::of::<T>();
        let boxed_any = retainers.remove(&type_id)?;
        let retainer: Retain<T> = *boxed_any.downcast().ok()?;

        retainers.insert(type_id, Box::new(retainer.clone()));
        self.0.replace(retainers);

        Some(retainer)
    }

    pub fn contains<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.0.borrow().contains_key(&type_id)
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
        retainers: &HandlerRetainers,
    ) -> Result<Self, Self::Error> {
        if retainers.contains::<T>() {
            let retainer = retainers.get::<T>().unwrap();
            Ok(retainer)
        } else {
            let retainer = T::build(message_data, retainers).ok().unwrap();

            retainers.retain(retainer);
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

pub trait IntoHandler<Marker>: Sized {
    fn into_handler(this: Self) -> FunctionHandler<Marker, Self>;
}

#[rustfmt::skip]
macro_rules! all_tuples {
    ($macro_name:ident) => {
        $macro_name!(T1);
        $macro_name!(T1, T2);
        $macro_name!(T1, T2, T3);
        $macro_name!(T1, T2, T3, T4);
        $macro_name!(T1, T2, T3, T4, T5);
        $macro_name!(T1, T2, T3, T4, T5, T6);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
        $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
    };
}

macro_rules! impl_handler {
   ($($ty:ident $(,)?)*)=> {
        #[allow(non_snake_case, unused_mut)]
        impl<$($ty,)* F> Handler for FunctionHandler<($($ty,)*), F>
        where
            $($ty: HandlerParam,)*
            F: FnMut($($ty,)*),
        {
            fn call(&mut self, message_data: crate::MessageData) {
                $(let $ty = $ty::build(message_data.clone(), &self.retainers).unwrap();)*
                (self.func)($($ty,)*);
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
        $macro_name!(T1, []);
        $macro_name!(T1, [T2]);
        $macro_name!(T1, [T2, T3]);
        $macro_name!(T1, [T2, T3, T4]);
        $macro_name!(T1, [T2, T3, T4, T5]);
        $macro_name!(T1, [T2, T3, T4, T5, T6]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15]);
        $macro_name!(T1, [T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16]);
    };
}

macro_rules! impl_into_handler {
    ($first:ident, [$($ty:ident $(,)?)*]) => {
        #[allow(non_snake_case, unused_mut)]
        impl<Msg, F, $first, $($ty,)*> IntoHandler<($first, $($ty,)*)> for F
        where
            Msg: crate::Message,
            $first: Deref<Target = Msg>,
            F: FnMut($first, $($ty,)*),
        {
            fn into_handler(this: Self) -> FunctionHandler<($first, $($ty,)*), Self> {
                FunctionHandler {
                    func: this,
                    marker: Default::default(),
                    message_type: Msg::TYPE_NAME.to_owned(),
                    retainers: Default::default(),
                }
            }
        }
    }
}

all_tuples_with_first!(impl_into_handler);
