use crate::{FunctionHandler, Handler, IntoHandler};

// Stores the handlers and resources/dependencies of those handlers
// "Consumes" a message, passing it to each handler that accepts that message type
pub struct Consumer<Executor> {
    handlers: Vec<Box<dyn Handler>>,
    category: Option<String>,
    identifier: Option<String>,
    poll_interval: u64,
    executor: Executor,
}

impl<Executor> Consumer<Executor> {
    pub fn new(executor: Executor) -> Self {
        Self {
            handlers: Vec::new(),
            category: None,
            identifier: None,
            poll_interval: 200,
            executor,
        }
    }

    pub fn add_handler<Params, Return, Func, T>(mut self, handler: T) -> Self
    where
        Executor: Clone + 'static,
        Params: 'static,
        Return: 'static,
        Func: IntoHandler<Params, Return, Func> + 'static,
        T: IntoHandler<Params, Return, Func> + 'static,
        FunctionHandler<Params, Return, Func>: Handler,
    {
        let handler: FunctionHandler<Params, Return, Func> =
            handler.insert_resource(self.executor.clone());

        let boxed_handler: Box<dyn Handler> = Box::new(handler);
        self.handlers.push(boxed_handler);

        self
    }

    pub fn category(mut self, category: &str) -> Self {
        self.category = Some(category.to_owned());
        self
    }

    pub fn identifier(mut self, identifier: &str) -> Self {
        self.identifier = Some(identifier.to_owned());
        self
    }

    pub fn poll_interval(mut self, poll_interval: u64) -> Self {
        self.poll_interval = poll_interval;
        self
    }
}
