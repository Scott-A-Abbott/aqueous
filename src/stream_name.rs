use std::collections::VecDeque;

pub mod separator {
    pub const ID: &'static str = "-";
    pub const CATEGORY_TYPE: &'static str = ":";
    pub const COMPOUND: &'static str = "+";
}

#[derive(Clone)]
pub struct StreamName(pub String);

impl StreamName {
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
        let (Category(category), id) = self.split();

        match id {
            Some(id) => {
                let mut ids = id.split();
                ids.push(new_id);

                let joined_id = StreamID::join(&ids);

                Self(category).add_id(joined_id)
            }
            None => {
                let StreamID(new_id) = new_id;
                let stream_name = format!("{}{}{}", category, separator::ID, new_id);

                Self(stream_name)
            }
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
        let category = Category(splits.pop_front().expect("Category").to_string());

        let id = if !splits.is_empty() {
            let id = StreamID(splits.make_contiguous().concat());
            Some(id)
        } else {
            None
        };

        (category, id)
    }
}

#[derive(Clone)]
pub struct StreamID(pub String);

impl StreamID {
    pub fn split(&self) -> Vec<StreamID> {
        let StreamID(id) = self;

        id.split(separator::COMPOUND)
            .map(|id| StreamID(id.to_string()))
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

#[derive(Clone)]
pub struct Category(pub String);

impl Category {
    pub fn split(&self) -> (EntityID, Option<CategoryType>) {
        let Self(category) = self;
        let mut splits = category.split(separator::ID).collect::<VecDeque<_>>();
        let entity_id = EntityID(splits.pop_front().expect("EntityID").to_string());

        let category_type = if !splits.is_empty() {
            let category_type = CategoryType(splits.make_contiguous().concat());
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
        let Self(category) = self;
        let StreamID(id) = id;
        let stream_name = format!("{}{}{}", category, separator::ID, id);

        StreamName(stream_name)
    }
}

#[derive(Clone)]
pub struct EntityID(pub String);

#[derive(Clone)]
pub struct CategoryType(pub String);

impl CategoryType {
    pub fn split(&self) -> Vec<CategoryType> {
        let CategoryType(category_type) = self;

        category_type
            .split(separator::COMPOUND)
            .map(|category_type| CategoryType(category_type.to_string()))
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
