use crate::{Message, MessageData, Msg};
use std::{marker::PhantomData, ops::Deref};

pub trait Handler<Executor, Settings> {
    fn call(&mut self, message_data: MessageData, executor: Executor, settings: Settings);
}

pub trait HandlerParam<Executor, Settings>: Sized {
    fn build(executor: Executor, settings: Settings) -> Self;
}

pub struct FunctionHandler<ExecutorMarker, ParamsMarker, ReturnMarker, F> {
    func: F,
    executor_marker: PhantomData<ExecutorMarker>,
    params_marker: PhantomData<ParamsMarker>,
    return_marker: PhantomData<ReturnMarker>,
}

pub trait IntoHandler<ExecutorMarker, ParamsMarker, ReturnMarker, Func>: Sized {
    fn into_handler(self) -> FunctionHandler<ExecutorMarker, ParamsMarker, ReturnMarker, Func>;
}

pub struct HandlerCollection<Params, Return, Executor, Settings> {
    pub handlers: Vec<Box<dyn Handler<Executor, Settings> + Send>>,
    params_marker: PhantomData<Params>,
    return_marker: PhantomData<Return>,
}

pub trait IntoHandlerCollection<Params, Return, Executor, Settings>: Sized {
    fn into_handler_collection(self) -> HandlerCollection<Params, Return, Executor, Settings>;
}

impl<F, Params, Return, Executor, Settings> IntoHandlerCollection<Params, Return, Executor, Settings> for F
where
    Params: Send + 'static,
    Executor: Send + 'static,
    Settings: Send + 'static,
    Return: std::future::Future<Output = ()> + Send + 'static,
    F: Func<Params, Return> + IntoHandler<Executor, Params, Return, F> + Send + 'static,
    FunctionHandler<Executor, Params, Return, F>: Handler<Executor, Settings>,
{
    fn into_handler_collection(self) -> HandlerCollection<Params, Return, Executor, Settings> {
        let function_handler = self.into_handler();
        let boxed_handler: Box<dyn Handler<Executor, Settings> + Send> = Box::new(function_handler);
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
        $macro_name!((T1, R1, F1));
        $macro_name!((T1, R1, F1), (T2, R2, F2));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7), (T8, R8, F8));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7), (T8, R8, F8), (T9, R9, F9));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7), (T8, R8, F8), (T9, R9, F9), (T10, R10, F10));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7), (T8, R8, F8), (T9, R9, F9), (T10, R10, F10), (T11, R11, F11));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7), (T8, R8, F8), (T9, R9, F9), (T10, R10, F10), (T11, R11, F11), (T12, R12, F12));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7), (T8, R8, F8), (T9, R9, F9), (T10, R10, F10), (T11, R11, F11), (T12, R12, F12), (T13, R13, F13));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7), (T8, R8, F8), (T9, R9, F9), (T10, R10, F10), (T11, R11, F11), (T12, R12, F12), (T13, R13, F13), (T14, R14, F14));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7), (T8, R8, F8), (T9, R9, F9), (T10, R10, F10), (T11, R11, F11), (T12, R12, F12), (T13, R13, F13), (T14, R14, F14), (T15, R15, F15));
        $macro_name!((T1, R1, F1), (T2, R2, F2), (T3, R3, F3), (T4, R4, F4), (T5, R5, F5), (T6, R6, F6), (T7, R7, F7), (T8, R8, F8), (T9, R9, F9), (T10, R10, F10), (T11, R11, F11), (T12, R12, F12), (T13, R13, F13), (T14, R14, F14), (T15, R15, F15), (T16, R16, F16));
    };
}

macro_rules! impl_into_handler_collection {
    ($(($param:ident, $re:ident, $fn:ident) $(,)?)*) => {
        #[allow(non_snake_case)]
        impl<Executor, Settings, $($param, $re, $fn),*> IntoHandlerCollection<($($param,)*), ($($re,)*), Executor, Settings> for ($($fn,)*)
        where
            $($fn: IntoHandlerCollection<$param, $re, Executor, Settings>),*
        {
            fn into_handler_collection(self) -> HandlerCollection<($($param,)*), ($($re,)*), Executor, Settings> {
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
        impl<F, $($ty,)* $re> Func<($($ty,)*), $re> for F where F: FnMut($($ty,)*) -> $re {}
    }
}

macro_rules! impl_into_handler_self {
    ([$($ty:ident $(,)?)*], $re:ident) => {
        #[allow(non_snake_case)]
        impl<Func, Executor, $($ty,)* $re> IntoHandler<Executor, ($($ty,)*), $re, Func> for FunctionHandler<Executor, ($($ty,)*), $re, Func>
        where
            Func: FnMut($($ty,)*) -> $re,
        {
            fn into_handler(self) -> Self {
                self
            }
        }
    }
}

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
        impl<$first, $($ty,)* $re, F, E, S> Handler<E, S> for FunctionHandler<E, (Msg<$first>, $($ty,)*), $re, F>
        where
            for<'de> $first: Message + serde::Deserialize<'de>,
            $($ty: HandlerParam<E, S>,)*
            F: FnMut(Msg<$first>, $($ty,)*) -> $re,
            $re: std::future::Future<Output = ()>,
            E: Clone + 'static,
            S: Clone,
        {
            fn call(&mut self, message_data: MessageData, _executor: E, _settings: S) {
                if message_data.type_name == $first::TYPE_NAME.to_string() {
                    let msg: Msg<$first> = Msg::from_data(message_data.clone()).unwrap();
                    $(let $ty = $ty::build(_executor.clone(), _settings.clone());)*
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
        #[allow(non_snake_case)]
        impl<E, Msg, Func, $first, $($ty,)* $re> IntoHandler<E, ($first, $($ty,)*), $re, Func> for Func
        where
            Msg: Message,
            $first: Deref<Target = Msg>,
            Func: FnMut($first, $($ty,)*) -> $re,
        {
            fn into_handler(self) -> FunctionHandler<E, ($first, $($ty,)*), $re, Self> {
                FunctionHandler {
                    func: self,
                    executor_marker: Default::default(),
                    params_marker: Default::default(),
                    return_marker: Default::default(),
                }
            }
        }
    }
}

function_params_with_first!(impl_handler);
function_params_with_first!(impl_into_handler);
