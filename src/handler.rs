// So far, this is based on the implementation of axum.
// However, if I were to model it to be more like bevy, it would look something like:
// struct FunctionHandler<Marker, F> where marker is the params (I think) and F is the function
// trait Handler with an initialize and call method and be implemented for FunctionHandler
// Trait IntoHandler that would be implemented for FnOnce(T,*) and convert to FunctionHandler
// The Consumer would contain some sort of list of Box<dyn Handler> that would be initialized from
// the consumer's state and message data

pub trait Handler<T> {
    fn call(self, message_data: crate::MessageData);
}

macro_rules! impl_handler {
    (
        $($ty:ident),*
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<Fn, $($ty,)*> Handler<($($ty,)*)> for Fn
        where
            Fn: FnOnce($($ty,)*),
            $( $ty: crate::FromConsumerState<Error = String>, )*
        {
            fn call(self, message_data: crate::MessageData) {
                $(
                    let $ty = match $ty::from_consumer_state(message_data.clone()) {
                        Ok(value) => value,
                        Err(_) => { return; }
                    };
                )*
                self($($ty,)*);
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! for_all_tuples {
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

for_all_tuples!(impl_handler);
