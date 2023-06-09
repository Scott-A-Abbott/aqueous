## Review
- EntityStore: `add_projections` that accepts an `IntoProjectionCollection`
- Simplify by removing the concept of an Executor, and using deadpool_postgres instead of sqlx
  - The pool would still need to be copied around to `HandlerParam::build`
