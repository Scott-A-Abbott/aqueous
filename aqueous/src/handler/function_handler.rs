use super::{Handler, HandlerParam};
use crate::message::{Message, MessageData, Msg};
use aqueous_macros::all_tuples;
use std::marker::PhantomData;

pub struct FunctionHandler<P, R, F> {
    func: F,
    params_marker: PhantomData<P>,
    return_marker: PhantomData<R>,
    message_type: &'static str,
}

impl<P, R, F> FunctionHandler<P, R, F> {
    pub fn message_type(&self) -> &'static str {
        self.message_type
    }
}

pub trait IntoHandler<P, R, F>: Sized {
    fn into_handler(self) -> FunctionHandler<P, R, F>;
}

macro_rules! impl_handler {
   ($($ty:ident $(,)?)*) => {
        #[allow(non_snake_case)]
        #[async_trait::async_trait]
        impl<M, $($ty,)* F, R, C, S> Handler<C, S> for FunctionHandler<(Msg<M>, $($ty,)*), R, F>
        where
            for<'de> M: Message + serde::Deserialize<'de> + Send,
            $($ty: HandlerParam<C, S> + Send,)*
            F: Fn(Msg<M>, $($ty,)*) -> R + Send,
            R: std::future::Future<Output = ()> + Send,
            C: Clone + Send + 'static,
            S: Clone + Send + 'static,
        {
            async fn call(&mut self, message_data: MessageData, _connection: C, _settings: S) -> bool {
                if &message_data.type_name == M::TYPE_NAME {
                    let msg: Msg<M> = Msg::from_data(message_data.clone()).unwrap();
                    $(let $ty = $ty::build(_connection.clone(), _settings.clone());)*
                    (self.func)(msg, $($ty,)*).await;
                    true
                } else {
                    false
                }
            }
        }
    }
}
all_tuples!(impl_handler, T);
