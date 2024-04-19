use super::{Handler, HandlerParam};
use crate::message::MessageData;
use aqueous_macros::all_tuples;
use std::marker::PhantomData;

pub struct CatchallFunctionHandler<P, R, F> {
    func: F,
    params_marker: PhantomData<P>,
    return_marker: PhantomData<R>,
}

pub trait IntoCatchallHandler<P, R, F>: Sized {
    fn into_catchall_handler(self) -> CatchallFunctionHandler<P, R, F>;
}

macro_rules! impl_into_catchall {
    ($($ty:ident $(,)?)*) => {
        impl<F, R, $($ty,)*> IntoCatchallHandler<(MessageData, $($ty,)*), R, F> for F
        where
            F: Fn(MessageData, $($ty,)*) -> R,
        {
            fn into_catchall_handler(self) -> CatchallFunctionHandler<(MessageData, $($ty),*), R, F> {
                CatchallFunctionHandler {
                    func: self,
                    params_marker: Default::default(),
                    return_marker: Default::default(),
                }
            }
        }
    }
}
all_tuples!(impl_into_catchall, T);

macro_rules! impl_handler_for_catchall {
   ($($ty:ident $(,)?)*) => {
        #[allow(non_snake_case)]
        #[async_trait::async_trait]
        impl<$($ty,)* F, R, C, S> Handler<C, S> for CatchallFunctionHandler<(MessageData, $($ty,)*), R, F>
        where
            $($ty: HandlerParam<C, S> + Send,)*
            F: Fn(MessageData, $($ty,)*) -> R + Send,
            R: std::future::Future<Output = ()> + Send,
            C: Clone + Send + 'static,
            S: Clone + Send + 'static,
        {
            async fn call(&mut self, message_data: MessageData, _connection: C, _settings: S) -> bool {
                $(let $ty = $ty::build(_connection.clone(), _settings.clone()).await;)*
                (self.func)(message_data, $($ty,)*).await;
                true
            }
        }
    }
}
all_tuples!(impl_handler_for_catchall, T);
