mod category;
pub mod separator;

pub use category::*;

use std::{
    collections::VecDeque,
    fmt::{Display, Formatter},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StreamName(pub String);

impl StreamName {
    pub fn new(stream_name: impl ToString) -> Self {
        Self(stream_name.to_string())
    }

    pub fn from_parts(category: Category, id: StreamID) -> Self {
        let Category(category) = category;
        let stream_name = category + separator::ID + id.as_ref();

        StreamName(stream_name)
    }

    pub fn has_id(&self) -> bool {
        let Self(stream_name) = self;
        stream_name.contains(separator::ID)
    }

    pub fn category(&self) -> Category {
        let (category, ..) = self.split();
        category
    }

    pub fn id(&self) -> Option<StreamID> {
        let (.., id) = self.split();
        id
    }

    pub fn ids(&self) -> Vec<StreamID> {
        let (.., id) = self.split();

        match id {
            Some(id) => id.split(),
            None => Vec::new(),
        }
    }

    pub fn add_id(&self, new_id: StreamID) -> Self {
        let (category, id) = self.split();

        match id {
            Some(id) => {
                let mut ids = id.split();
                ids.push(new_id);

                let joined_id = StreamID::join(&ids);

                Self::from_parts(category, joined_id)
            }
            None => Self::from_parts(category, new_id),
        }
    }

    pub fn cardinal_id(&self) -> Option<StreamID> {
        let (.., id) = self.split();
        let ids = id?.split();
        let id = ids.first()?;

        Some(id.clone())
    }

    pub fn split(&self) -> (Category, Option<StreamID>) {
        let Self(stream_name) = self;
        let mut splits = stream_name.split(separator::ID).collect::<VecDeque<_>>();
        let category = Category::new(splits.pop_front().expect("Category"));

        let id = if !splits.is_empty() {
            let id = StreamID(splits.make_contiguous().concat());
            Some(id)
        } else {
            None
        };

        (category, id)
    }
}

impl AsRef<str> for StreamName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for StreamName {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct StreamID(pub String);

impl StreamID {
    pub fn new(stream_id: impl ToString) -> Self {
        Self(stream_id.to_string())
    }

    pub fn split(&self) -> Vec<StreamID> {
        let StreamID(stream_id_string) = self;

        stream_id_string
            .split(separator::COMPOUND)
            .map(StreamID::new)
            .collect()
    }

    pub fn join(ids: &[Self]) -> Self {
        let joined_ids = ids
            .iter()
            .map(|Self(id)| id.as_str())
            .collect::<Vec<_>>()
            .join(separator::COMPOUND);

        Self(joined_ids)
    }
}

impl AsRef<str> for StreamID {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for StreamID {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
