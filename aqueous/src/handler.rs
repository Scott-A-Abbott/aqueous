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

macro_rules! impl_into_handler_self {
    ([$($ty:ident $(,)?)*], $re:ident)=> {
        #[allow(non_snake_case, unused_mut)]
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
   ($first:ident, [$($ty:ident $(,)?)*], $re:ident)=> {
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
        #[allow(non_snake_case, unused_mut)]
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
