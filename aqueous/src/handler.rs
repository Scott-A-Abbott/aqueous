mod catchall_handler;
mod function_handler;
mod handler_collection;

pub use catchall_handler::*;
pub use function_handler::*;
pub use handler_collection::*;

use crate::message::MessageData;
use async_trait::async_trait;

#[async_trait]
pub trait Handler<C, S> {
    async fn call(&mut self, message_data: MessageData, connection: C, settings: S) -> bool;
}

#[async_trait]
pub trait HandlerParam<C, S = ()>: Sized {
    async fn build(connection: C, settings: S) -> Self;
}
