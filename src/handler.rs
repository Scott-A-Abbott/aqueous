pub trait Handler<T> {
    fn call(self, message_data: crate::MessageData);
}

impl<F> Handler<((),)> for F
where
    F: FnOnce() + 'static,
{
    fn call(self, _: crate::MessageData) {
        self();
    }
}

macro_rules! impl_handler {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<F, $($ty,)* $last> Handler<($($ty,)* $last,)> for F
        where
            F: FnOnce($($ty,)* $last,) + 'static,
            $( $ty: crate::FromConsumerState<Error = String>, )*
            $last: crate::FromConsumerState<Error = String>,
        {
            fn call(self, message_data: crate::MessageData) {
                $(
                    let $ty = $ty::from_consumer_state(message_data.clone()).unwrap();
                )*

                let $last = $last::from_consumer_state(message_data.clone()).unwrap();

                self($($ty,)* $last,);
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! for_all_tuples {
    ($name:ident) => {
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

for_all_tuples!(impl_handler);
