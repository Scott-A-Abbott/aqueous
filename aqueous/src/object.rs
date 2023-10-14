use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum Object<A, S> {
    Actuator(A),
    Substitute(Arc<Mutex<S>>),
}

impl<A, S> Object<A, S> {
    pub fn substitute(substitute: S) -> Self {
        let substitute_mutex = Mutex::new(substitute);
        let substitute_arc = Arc::new(substitute_mutex);

        Self::Substitute(substitute_arc)
    }

    pub fn unwrap_substitute(self) -> S
    where
        S: std::fmt::Debug,
    {
        let Object::Substitute(sub) = self else { panic!("Attempted to unwrap an actuator") };
        let sub = Arc::try_unwrap(sub).expect("Unwrap substitute");
        sub.into_inner().unwrap()
    }
}
