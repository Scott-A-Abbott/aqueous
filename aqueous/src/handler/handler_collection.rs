use super::Handler;
use aqueous_macros::all_tuples;
use std::{collections::HashMap, marker::PhantomData};

pub struct HandlerCollection<P, R, C, S = ()> {
    params_marker: PhantomData<P>,
    return_marker: PhantomData<R>,
    pub handlers: HashMap<&'static str, Box<dyn Handler<C, S> + Send>>,
}

pub trait IntoHandlerCollection<P, R, C, S = ()>: Sized {
    fn into_handler_collection(self) -> HandlerCollection<P, R, C, S>;
}

macro_rules! impl_into_handler_collection {
    ($(($param:ident, $re:ident, $fn:ident) $(,)?)*) => {
        #[allow(non_snake_case, unused_mut)]
        impl<C, S, $($param, $re, $fn),*> IntoHandlerCollection<($($param,)*), ($($re,)*), C, S> for ($($fn,)*)
        where
            $($fn: IntoHandlerCollection<$param, $re, C, S>),*
        {
            fn into_handler_collection(self) -> HandlerCollection<($($param,)*), ($($re,)*), C, S> {
                let ($($fn,)*) = self;
                let mut handlers = HashMap::new();

                $( let HandlerCollection { handlers: $fn, .. } = $fn.into_handler_collection(); )*
                $( handlers.extend($fn.into_iter()); )*

                HandlerCollection {
                    handlers,
                    params_marker: Default::default(),
                    return_marker: Default::default(),
                }
            }
        }
    }
}

all_tuples!(impl_into_handler_collection, P, R, F);
