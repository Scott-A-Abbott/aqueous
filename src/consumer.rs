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
