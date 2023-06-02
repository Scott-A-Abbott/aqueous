## Consumer
- The Consumer starts an Actor and Subscription
  - The Actor asks the Subscription for messages
  - The Subscription polls the store for messages and sends them to the Actor in batch
  - The Actor dispatches the messages to the Consumer
  - The Consumer processes each message with it's collection of Handlers
- Look into bounded channels for communication between parts, with the bound being the batch size

## Handler
- The eventide handlers don't persist data, the are built fresh each time
- The primary use case I had for persisting were things like the Store
  - An alternative to persisting the store would be to give it a handle to some kind of external cache
  - Something like the `Moka` crate
  - I believe eventide also allows for a way to persist the cache to the message store...
- I would also need to ensure that the `Executor` (session in the evt world) is passed in, instead of the handler resources
  - This could be done during `Handler::call`
