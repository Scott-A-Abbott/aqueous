use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum UsefulObject<A, S> {
    Actuator(A),
    Substitute(Arc<Mutex<S>>),
}

impl<A, S> UsefulObject<A, S> {
    pub fn substitute(substitute: S) -> Self {
        let substitute_cell = Mutex::new(substitute);
        let substitute_rc = Arc::new(substitute_cell);

        Self::Substitute(substitute_rc)
    }

    pub fn unwrap_substitute(self) -> S
    where
        S: std::fmt::Debug,
    {
        let UsefulObject::Substitute(sub) = self else { panic!("Attempted to unwrap a actuator") };
        let sub = Arc::try_unwrap(sub).expect("Unwrap substitute");
        sub.into_inner().unwrap()
    }
}
