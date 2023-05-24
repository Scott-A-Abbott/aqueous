// Stores the handlers and resources/dependencies of those handlers
// "Consumes" a message, passing it to each handler that accepts that message type
pub struct Consumer {
    handlers: Vec<()>,
    resources: Vec<()>,
    dependencies: Vec<()>,
    category: String,
    identifier: String,
    poll_interval: u64,
}

impl Consumer {
    // This will need to convert a handler function into a Handler
    // with HandlerParams that map to messages, resources, etc.
    // TODO: get inspiration from bevy or axum for how this should work
    pub fn with_handler(handler: impl Fn()) {}
    pub fn with_resource<T>(resource: T) {}
}

// Consider borrowing from bevy to create a FunctionHandler struct that keeps track of it's own state
pub trait FromConsumerState: Sized {
    type Error;

    fn from_consumer_state(message_data: crate::message::MessageData) -> Result<Self, Self::Error>;
}

pub trait Handler<T> {
    fn call(self, message_data: crate::message::MessageData);
}

impl<F> Handler<((),)> for F
where
    F: FnOnce() + 'static,
{
    fn call(self, _: crate::message::MessageData) {
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
            $( $ty: FromConsumerState<Error = String>, )*
            $last: FromConsumerState<Error = String>,
        {
            fn call(self, message_data: crate::message::MessageData) {
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
