## Dependencies
- Should _all_ `HandlerParam`s be retained?
  - If so, how should that be implemented?
  - Currently, the `Retain` wrapper is a smart pointer. This enables it to be cloned into the handler when called (with interior mutability)
- There may be a use case for shared dependencies between handlers
  - In that case, maybe the consumer has a `shared_dependency` like function that inserts a retain into each handler that is passed in

## Consumer
- The Consumer starts an Actor and Subscription
  - The Actor asks the Subscription for messages
  - The Subscription polls the store for messages and sends them to the Actor in batch
  - The Actor dispatches the messages to the Consumer
  - The Consumer processes each message with it's collection of Handlers
- Look into bounded channels for communication between parts, with the bound being the batch size
