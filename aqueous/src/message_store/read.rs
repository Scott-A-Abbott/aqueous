use crate::{
    message_store::{
        get::{GetCategory, GetLast, GetStream},
        Connection, Error,
    },
    stream_name::StreamName,
    MessageData, Object,
};

pub struct Read {
    object: Object<ReadMessages, ReadSubstitute>,
}

impl Read {
    pub fn new(substitute: ReadSubstitute) -> Self {
        let object = Object::substitute(substitute);
        Self { object }
    }

    pub fn build(connection: Connection) -> Self {
        let actuator = ReadMessages::build(connection);
        let object = Object::Actuator(actuator);
        Self { object }
    }

    pub fn unwrap_substitute(self) -> ReadSubstitute {
        self.object.unwrap_substitute()
    }

    pub async fn execute(&mut self, stream_name: impl ToString) -> Result<Vec<MessageData>, Error> {
        let stream_name = StreamName::new(stream_name);

        use Object::*;
        match &mut self.object {
            Actuator(actuator) => actuator.execute(stream_name).await,
            Substitute(substitute) => {
                let mut substitute = substitute.lock().unwrap();
                substitute.execute(stream_name).await
            }
        }
    }

    // pub position: Option<i64>,
    pub fn position(&mut self, position: i64) -> &mut Self {
        let position = Some(position);

        use Object::*;
        match &mut self.object {
            Actuator(actuator) => actuator.options.position = position,
            Substitute(substitute) => {
                let mut substitute = substitute.lock().unwrap();
                substitute.options.position = position;
            }
        }

        self
    }

    pub fn batch_size(&mut self, batch_size: i64) -> &mut Self {
        let batch_size = Some(batch_size);

        use Object::*;
        match &mut self.object {
            Actuator(actuator) => actuator.options.batch_size = batch_size,
            Substitute(substitute) => {
                let mut substitute = substitute.lock().unwrap();
                substitute.options.batch_size = batch_size;
            }
        }

        self
    }

    pub fn condition(&mut self, condition: &str) -> &mut Self {
        let condition = Some(condition.to_string());

        use Object::*;
        match &mut self.object {
            Actuator(actuator) => actuator.options.condition = condition,
            Substitute(substitute) => {
                let mut substitute = substitute.lock().unwrap();
                substitute.options.condition = condition;
            }
        }

        self
    }

    pub fn message_type(&mut self, message_type: &str) -> &mut Self {
        let message_type = Some(message_type.to_string());

        use Object::*;
        match &mut self.object {
            Actuator(actuator) => actuator.options.message_type = message_type,
            Substitute(substitute) => {
                let mut substitute = substitute.lock().unwrap();
                substitute.options.message_type = message_type;
            }
        }

        self
    }

    pub fn stream_name(&mut self, stream_name: impl ToString) -> &mut Self {
        let stream_name = Some(StreamName::new(stream_name));

        use Object::*;
        match &mut self.object {
            Actuator(actuator) => actuator.options.stream_name = stream_name,
            Substitute(substitute) => {
                let mut substitute = substitute.lock().unwrap();
                substitute.options.stream_name = stream_name;
            }
        }

        self
    }

    pub fn last(&mut self, last: bool) -> &mut Self {
        use Object::*;
        match &mut self.object {
            Actuator(actuator) => actuator.options.last = last,
            Substitute(substitute) => {
                let mut substitute = substitute.lock().unwrap();
                substitute.options.last = last;
            }
        }

        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReadOptions {
    pub position: Option<i64>,
    pub batch_size: Option<i64>,
    pub condition: Option<String>,
    pub message_type: Option<String>,
    pub stream_name: Option<StreamName>,
    pub last: bool,
}

pub struct ReadMessages {
    connection: Connection,
    options: ReadOptions,
}

impl ReadMessages {
    pub fn build(connection: Connection) -> Self {
        Self {
            connection,
            options: ReadOptions::default(),
        }
    }

    pub async fn execute(&mut self, stream_name: StreamName) -> Result<Vec<MessageData>, Error> {
        self.options.stream_name = Some(stream_name.clone());

        if self.options.last {
            return self.get_last().await;
        }

        let is_category = !stream_name.has_id();
        if is_category {
            return self.get_category().await;
        }

        self.get_stream().await
    }

    async fn get_last(&mut self) -> Result<Vec<MessageData>, Error> {
        let stream_name = self.options.stream_name.take().unwrap();

        let mut get_last = GetLast::new(self.connection.clone());

        let optional_message_type = self.options.message_type.clone();
        if let Some(message_type) = optional_message_type {
            get_last.message_type(&message_type);
        }

        let messages = get_last
            .execute(stream_name)
            .await?
            .map(|message| vec![message])
            .unwrap_or_default();

        Ok(messages)
    }

    async fn get_category(&mut self) -> Result<Vec<MessageData>, Error> {
        let mut get_category = GetCategory::new(self.connection.clone());

        let position = self.options.position.clone();
        if let Some(position) = position {
            get_category.position(position);
        }

        let batch_size = self.options.batch_size.clone();
        if let Some(batch_size) = batch_size {
            get_category.batch_size(batch_size);
        }

        if let Some(condition) = self.condition() {
            get_category.condition(&condition);
        }

        let stream_name = self.options.stream_name.clone().unwrap();
        let category = stream_name.category();

        get_category.execute(category).await
    }

    async fn get_stream(&mut self) -> Result<Vec<MessageData>, Error> {
        let mut get_stream = GetStream::new(self.connection.clone());

        let position = self.options.position.clone();
        if let Some(position) = position {
            get_stream.position(position);
        }

        let batch_size = self.options.batch_size.clone();
        if let Some(batch_size) = batch_size {
            get_stream.batch_size(batch_size);
        }

        let condition = self.condition();
        if let Some(condition) = condition.as_ref() {
            get_stream.condition(condition);
        }

        let stream_name = self.options.stream_name.clone().unwrap();
        get_stream.execute(stream_name).await
    }

    fn condition(&self) -> Option<String> {
        let condition = self.options.condition.clone();
        let message_type = self.options.message_type.clone();

        condition
            .map(|condition| {
                if let Some(message_type) = message_type.clone() {
                    return format!("{} AND type = {}", condition, message_type);
                }

                condition
            })
            .or_else(|| message_type.map(|message_type| format!("type = {}", message_type)))
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReadSubstitute {
    pub error: Option<Error>,
    pub message_data: Option<MessageData>,
    pub options: ReadOptions,
}

impl ReadSubstitute {
    async fn execute(&mut self, stream_name: StreamName) -> Result<Vec<MessageData>, Error> {
        let read_options = &mut self.options;
        read_options.stream_name = Some(stream_name);

        let optional_error = self.error.take();
        let optional_message_data = self.message_data.take();

        match optional_error {
            Some(error) => Err(error),
            None => {
                if let Some(message_data) = optional_message_data {
                    return Ok(vec![message_data]);
                }

                Ok(Vec::new())
            }
        }
    }
}
