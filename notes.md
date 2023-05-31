## Dependencies
- Should _all_ `HandlerParam`s be retained?
  - If so, how should that be implemented?
  - Currently, the `Retain` wrapper is a smart pointer. This enables it to be cloned into the handler when called (with interior mutability)
- There may be a use case for shared dependencies between handlers
  - In that case, maybe the consumer has a `shared_dependency` like function that inserts a retain into each handler that is passed in
