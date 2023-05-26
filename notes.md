- Is there a use case for state dependent dependencies?
  - In other words, a dependency that should persist between handler invocations
  - Would they need to be differentiated from others?
  - Receiving a DB connection from a pool, for example, maybe shouldn't be persisted; to free up the connection

- How should `Consumer` state be handled?
  - I imagine there would be state that it has by design, such as the message store
  - User defined state would be used for shared configuration/dependencies of handlers
