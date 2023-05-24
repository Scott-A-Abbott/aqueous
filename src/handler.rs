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
            Fn: FnOnce($($ty,)*) + 'static,
            $( $ty: crate::FromConsumerState<Error = String>, )*
        {
            fn call(self, message_data: crate::MessageData) {
                $(
                    let $ty = $ty::from_consumer_state(message_data.clone()).unwrap();
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
