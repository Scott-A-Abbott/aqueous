use crate::{Message, MessageData, Msg};
use sqlx::PgPool;
use std::{marker::PhantomData, ops::Deref};

pub trait Handler<Settings = ()> {
    fn call(&mut self, message_data: MessageData, pool: PgPool, settings: Settings) -> bool;
}

pub trait HandlerParam<Settings = ()>: Sized {
    fn build(pool: PgPool, settings: Settings) -> Self;
}

pub struct FunctionHandler<Params, Return, F> {
    func: F,
    params_marker: PhantomData<Params>,
    return_marker: PhantomData<Return>,
}

pub trait IntoHandler<Params, Return, Func>: Sized {
    fn into_handler(self) -> FunctionHandler<Params, Return, Func>;
}

pub struct CatchallFunctionHandler<Params, Return, F> {
    func: F,
    params_marker: PhantomData<Params>,
    return_marker: PhantomData<Return>,
}

pub trait IntoCatchallHandler<Params, Return, Func>: Sized {
    fn into_catchall_handler(self) -> CatchallFunctionHandler<Params, Return, Func>;
}

pub struct HandlerCollection<Params, Return, Settings = ()> {
    pub handlers: Vec<Box<dyn Handler<Settings> + Send>>,
    params_marker: PhantomData<Params>,
    return_marker: PhantomData<Return>,
}

pub trait IntoHandlerCollection<Params, Return, Settings = ()>: Sized {
    fn into_handler_collection(self) -> HandlerCollection<Params, Return, Settings>;
}

impl<F, Params, Return, Settings> IntoHandlerCollection<Params, Return, Settings> for F
where
    Params: Send + 'static,
    Settings: Send + 'static,
    Return: std::future::Future<Output = ()> + Send + 'static,
    F: Func<Params, Return> + IntoHandler<Params, Return, F> + Send + 'static,
    FunctionHandler<Params, Return, F>: Handler<Settings>,
{
    fn into_handler_collection(self) -> HandlerCollection<Params, Return, Settings> {
        let function_handler = self.into_handler();
        let boxed_handler: Box<dyn Handler<Settings> + Send> = Box::new(function_handler);
        let handlers = vec![boxed_handler];

        HandlerCollection {
            handlers,
            params_marker: Default::default(),
            return_marker: Default::default(),
        }
    }
}

pub trait Func<Params, Return> {}

#[rustfmt::skip]
macro_rules! all_function_tuples {
    ($macro_name:ident) => {
        $macro_name!((P1, R1, F1));
        $macro_name!((P1, R1, F1), (P2, R2, F2));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7), (P8, R8, F8));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7), (P8, R8, F8), (P9, R9, F9));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7), (P8, R8, F8), (P9, R9, F9), (P10, R10, F10));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7), (P8, R8, F8), (P9, R9, F9), (P10, R10, F10), (P11, R11, F11));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7), (P8, R8, F8), (P9, R9, F9), (P10, R10, F10), (P11, R11, F11), (P12, R12, F12));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7), (P8, R8, F8), (P9, R9, F9), (P10, R10, F10), (P11, R11, F11), (P12, R12, F12), (P13, R13, F13));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7), (P8, R8, F8), (P9, R9, F9), (P10, R10, F10), (P11, R11, F11), (P12, R12, F12), (P13, R13, F13), (P14, R14, F14));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7), (P8, R8, F8), (P9, R9, F9), (P10, R10, F10), (P11, R11, F11), (P12, R12, F12), (P13, R13, F13), (P14, R14, F14), (P15, R15, F15));
        $macro_name!((P1, R1, F1), (P2, R2, F2), (P3, R3, F3), (P4, R4, F4), (P5, R5, F5), (P6, R6, F6), (P7, R7, F7), (P8, R8, F8), (P9, R9, F9), (P10, R10, F10), (P11, R11, F11), (P12, R12, F12), (P13, R13, F13), (P14, R14, F14), (P15, R15, F15), (P16, R16, F16));
    };
}

macro_rules! impl_into_handler_collection {
    ($(($param:ident, $re:ident, $fn:ident) $(,)?)*) => {
        #[allow(non_snake_case)]
        impl<Settings, $($param, $re, $fn),*> IntoHandlerCollection<($($param,)*), ($($re,)*), Settings> for ($($fn,)*)
        where
            $($fn: IntoHandlerCollection<$param, $re, Settings>),*
        {
            fn into_handler_collection(self) -> HandlerCollection<($($param,)*), ($($re,)*), Settings> {
                let ($($fn,)*) = self;
                let mut handlers = Vec::new();

                $( let HandlerCollection { handlers: mut $fn, .. } = $fn.into_handler_collection(); )*
                $( handlers.append(&mut $fn); )*

                HandlerCollection {
                    handlers,
                    params_marker: Default::default(),
                    return_marker: Default::default(),
                }
            }
        }
    }
}

all_function_tuples!(impl_into_handler_collection);

#[rustfmt::skip]
macro_rules! function_params {
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

macro_rules! impl_func {
    ([$($ty:ident $(,)?)*], $re:ident) => {
        impl<F, $($ty,)* $re> Func<($($ty,)*), $re> for F where F: Fn($($ty,)*) -> $re {}
    }
}

macro_rules! impl_into_handler_self {
    ([$($ty:ident $(,)?)*], $re:ident) => {
        #[allow(non_snake_case)]
        impl<Func, $($ty,)* $re> IntoHandler<($($ty,)*), $re, Func> for FunctionHandler<($($ty,)*), $re, Func>
        where
            Func: Fn($($ty,)*) -> $re,
        {
            fn into_handler(self) -> Self {
                self
            }
        }
    }
}

macro_rules! impl_into_catchall {
    ([$($ty:ident $(,)?)*], $re:ident) => {
        impl<Func, $($ty,)* $re> IntoCatchallHandler<(MessageData, $($ty,)*), $re, Func> for Func
        where
            Func: Fn(MessageData, $($ty,)*) -> $re,
        {
            fn into_catchall_handler(self) -> CatchallFunctionHandler<(MessageData, $($ty),*), $re, Func> {
                CatchallFunctionHandler {
                    func: self,
                    params_marker: Default::default(),
                    return_marker: Default::default(),
                }
            }
        }
    }
}
impl_into_catchall!([], R);

macro_rules! impl_handler_for_catchall {
   ([$($ty:ident $(,)?)*], $re:ident) => {
        #[allow(non_snake_case)]
        impl<$($ty,)* $re, F, S> Handler<S> for CatchallFunctionHandler<(MessageData, $($ty,)*), $re, F>
        where
            $($ty: HandlerParam<S>,)*
            F: Fn(MessageData, $($ty,)*) -> $re,
            $re: std::future::Future<Output = ()>,
            S: Clone,
        {
            fn call(&mut self, message_data: MessageData, _pool: PgPool, _settings: S) -> bool {
                $(let $ty = $ty::build(_pool.clone(), _settings.clone());)*
                tokio::task::block_in_place(move || {
                    tokio::runtime::Handle::current().block_on(async move {
                        (self.func)(message_data, $($ty,)*).await;
                        true
                    })
                })
            }
        }
    }
}
impl_handler_for_catchall!([], R);

function_params!(impl_into_catchall);
function_params!(impl_handler_for_catchall);
function_params!(impl_into_handler_self);
function_params!(impl_func);

#[rustfmt::skip]
macro_rules! function_params_with_first {
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
   ($first:ident, [$($ty:ident $(,)?)*], $re:ident) => {
        #[allow(non_snake_case)]
        impl<$first, $($ty,)* $re, F, S> Handler<S> for FunctionHandler<(Msg<$first>, $($ty,)*), $re, F>
        where
            for<'de> $first: Message + serde::Deserialize<'de>,
            $($ty: HandlerParam<S>,)*
            F: Fn(Msg<$first>, $($ty,)*) -> $re,
            $re: std::future::Future<Output = ()>,
            S: Clone,
        {
            fn call(&mut self, message_data: MessageData, _pool: PgPool, _settings: S) -> bool {
                if message_data.type_name == $first::TYPE_NAME.to_string() {
                    let msg: Msg<$first> = Msg::from_data(message_data.clone()).unwrap();
                    $(let $ty = $ty::build(_pool.clone(), _settings.clone());)*
                    tokio::task::block_in_place(move || {
                        tokio::runtime::Handle::current().block_on(async move {
                            (self.func)(msg, $($ty,)*).await;
                            true
                        })
                    })
                } else {
                    false
                }
            }
        }
    }
}

macro_rules! impl_into_handler {
    ($first:ident, [$($ty:ident $(,)?)*], $re:ident) => {
        #[allow(non_snake_case)]
        impl<Msg, Func, $first, $($ty,)* $re> IntoHandler<($first, $($ty,)*), $re, Func> for Func
        where
            Msg: Message,
            $first: Deref<Target = Msg>,
            Func: Fn($first, $($ty,)*) -> $re,
        {
            fn into_handler(self) -> FunctionHandler<($first, $($ty,)*), $re, Self> {
                FunctionHandler {
                    func: self,
                    params_marker: Default::default(),
                    return_marker: Default::default(),
                }
            }
        }
    }
}

function_params_with_first!(impl_handler);
function_params_with_first!(impl_into_handler);
