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
    pub fn with_handler(handler: impl crate::Handler) {}
    pub fn with_resource<Res>(resource: Res) {}
}
