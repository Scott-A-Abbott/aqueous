use std::marker::PhantomData;

pub trait Handler {
    fn call(&mut self, message_data: crate::MessageData);
}

pub trait HandlerParam: Sized {
    type Error: std::fmt::Debug;

    fn initialize(message_data: crate::MessageData) -> Result<Self, Self::Error>;
}

pub struct FunctionHandler<Marker, F> {
    func: F,
    marker: PhantomData<Marker>,
}

impl<T1, F> Handler for FunctionHandler<T1, F>
where
    T1: HandlerParam,
    F: FnMut(T1),
{
    fn call(&mut self, message_data: crate::MessageData) {
        let t1 = T1::initialize(message_data).unwrap();
        (self.func)(t1);
    }
}

pub trait IntoHandler<Marker>: Sized {
    fn into_handler(this: Self) -> FunctionHandler<Marker, Self>;
}

impl<T1, F> IntoHandler<T1> for F
where
    F: FnMut(T1),
{
    fn into_handler(this: Self) -> FunctionHandler<T1, Self> {
        FunctionHandler {
            func: this,
            marker: Default::default(),
        }
    }
}

// #[rustfmt::skip]
// macro_rules! for_all_tuples {
//     ($macro_name:ident) => {
//         $macro_name!(T1);
//         $macro_name!(T1, T2);
//         $macro_name!(T1, T2, T3);
//         $macro_name!(T1, T2, T3, T4);
//         $macro_name!(T1, T2, T3, T4, T5);
//         $macro_name!(T1, T2, T3, T4, T5, T6);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
//         $macro_name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
//     };
// }
