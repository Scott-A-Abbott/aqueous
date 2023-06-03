## Consumer
- The Consumer starts an Actor and Subscription
  - The Actor asks the Subscription for messages
  - The Subscription polls the store for messages and sends them to the Actor in batch
  - The Actor dispatches the messages to the Consumer
  - The Consumer processes each message with it's collection of Handlers
- Look into bounded channels for communication between parts, with the bound being the batch size
