mod message_data;
mod metadata;
mod msg;

pub use message_data::*;
pub use metadata::*;
pub use msg::*;

pub trait Message: Sized {
    const TYPE_NAME: &'static str;

    fn type_name(&self) -> &'static str {
        Self::TYPE_NAME
    }
}
