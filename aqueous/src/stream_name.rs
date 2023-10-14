use std::{
    collections::VecDeque,
    fmt::{Display, Formatter},
};

pub mod separator {
    pub const ID: &'static str = "-";
    pub const CATEGORY_TYPE: &'static str = ":";
    pub const COMPOUND: &'static str = "+";
}

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
            .map(|stream_id_str| StreamID::new(stream_id_str))
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

#[derive(Clone, Debug)]
pub struct Category(pub String);

impl Category {
    pub fn new(category: impl ToString) -> Self {
        Self(category.to_string())
    }

    pub fn from_parts(entity_id: EntityID, category_type: CategoryType) -> Self {
        let EntityID(entity_id) = entity_id;
        let CategoryType(category_type) = category_type;
        let category_string = entity_id + separator::CATEGORY_TYPE + &category_type;

        Self::new(category_string)
    }

    pub fn new_command(category: impl ToString) -> Self {
        let category_type = CategoryType::new("command");

        Self::new(category).add_type(category_type)
    }

    pub fn split(&self) -> (EntityID, Option<CategoryType>) {
        let Self(category) = self;
        let mut splits = category
            .split(separator::CATEGORY_TYPE)
            .collect::<VecDeque<_>>();
        let id_str = splits.pop_front().expect("EntityID");
        let entity_id = EntityID::new(id_str);

        let category_type = if !splits.is_empty() {
            let type_string = splits.make_contiguous().concat();
            let category_type = CategoryType(type_string);

            Some(category_type)
        } else {
            None
        };

        (entity_id, category_type)
    }

    pub fn entity_id(&self) -> EntityID {
        let (entity_id, ..) = self.split();
        entity_id
    }

    pub fn category_type(&self) -> Option<CategoryType> {
        let (.., category_type) = self.split();
        category_type
    }

    pub fn category_types(&self) -> Vec<CategoryType> {
        let (.., category_type) = self.split();

        match category_type {
            Some(category_type) => category_type.split(),
            None => Vec::new(),
        }
    }

    pub fn has_type(&self) -> bool {
        let Category(category) = self;

        category.contains(separator::CATEGORY_TYPE)
    }

    pub fn add_type(&self, new_type: CategoryType) -> Self {
        let (EntityID(entity_id), category_type) = self.split();

        match category_type {
            Some(category_type) => {
                let mut types = category_type.split();
                types.push(new_type);

                let joined_type = CategoryType::join(&types);

                Self(entity_id).add_type(joined_type)
            }
            None => {
                let CategoryType(new_type) = new_type;
                let category = format!("{}{}{}", entity_id, separator::CATEGORY_TYPE, new_type);

                Self(category)
            }
        }
    }

    pub fn stream_name(&self, id: StreamID) -> StreamName {
        StreamName::from_parts(self.clone(), id)
    }
}

impl AsRef<str> for Category {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct EntityID(pub String);

impl EntityID {
    pub fn new(entity_id: impl ToString) -> Self {
        Self(entity_id.to_string())
    }
}

impl AsRef<str> for EntityID {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for EntityID {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct CategoryType(pub String);

impl CategoryType {
    pub fn new(category_type: impl ToString) -> Self {
        Self(category_type.to_string())
    }

    pub fn split(&self) -> Vec<CategoryType> {
        let CategoryType(category_type_string) = self;

        category_type_string
            .split(separator::COMPOUND)
            .map(|category_type_str| CategoryType::new(category_type_str))
            .collect()
    }

    pub fn join(types: &[Self]) -> Self {
        let joined_type = types
            .iter()
            .map(|Self(s)| s.as_str())
            .collect::<Vec<_>>()
            .join(separator::COMPOUND);

        Self(joined_type)
    }
}

impl AsRef<str> for CategoryType {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for CategoryType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
