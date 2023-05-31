## `Store` and `Projection`
- [observation] There aren't any projections that have dependencies
  - Is this coincidence? Or would it be a "smell" if they did?
- Just as with the message store objects, a pool could be passed in to fetch it's own connection
- The store should eventually be able to persist the entity state and where it has read to
  - Would it be acceptable to calculate the entity state from the beginning, for now?
- Should the `Projection` actually work similarly to a `HandlerFunction`?
  - Something like `|entity: &mut AccountEntity, deposited: Msg<Deposited>| { //do stuff }`

## Dependencies
- Should _all_ `HandlerParam`s be retained?
  - If so, how should that be implemented?
  - Currently, the `Retain` wrapper is a smart pointer. This enables it to be cloned into the handler when called (with interior mutability)
