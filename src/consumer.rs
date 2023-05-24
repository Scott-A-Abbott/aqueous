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
    pub fn with_handler<Params>(handler: impl crate::Handler<Params>) {}
    pub fn with_resource<Res>(resource: Res) {}
}

// Consider borrowing from bevy to create a FunctionHandler struct that keeps track of it's own state
pub trait FromConsumerState: Sized {
    type Error;

    fn from_consumer_state(message_data: crate::MessageData) -> Result<Self, Self::Error>;
}
