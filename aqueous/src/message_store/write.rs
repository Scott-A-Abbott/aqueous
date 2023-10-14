use crate::*;
use serde_json::Value;
use sqlx::Execute;
use tracing::{instrument, trace};

#[derive(Clone, Default)]
pub struct Write {
    object: Object<WriteMessages, WriteSubstitute>,
}

impl Write {
    pub fn new(substitute: WriteSubstitute) -> Self {
        let object = Object::substitute(substitute);
        Self { object }
    }

    pub fn build(connection: Connection) -> Self {
        let actuator = WriteMessages::build(connection);
        let object = Object::Actuator(actuator);
        Self { object }
    }

    pub fn unwrap_substitute(self) -> WriteSubstitute {
        self.object.unwrap_substitute()
    }

    pub fn expected_version(&mut self, expected_version: Version) -> &mut Self {
        match &mut self.object {
            Object::Actuator(actuator) => {
                actuator.expected_version = Some(expected_version);
            }
            Object::Substitute(substitute) => {
                substitute.lock().unwrap().expected_version = Some(expected_version);
            }
        }

        self
    }

    pub fn add_message<T>(&mut self, message: impl Into<Msg<T>>) -> &mut Self
    where
        T: serde::Serialize + Message,
    {
        let msg: Msg<T> = message.into();
        let msg_data = msg.try_into().unwrap();

        match &mut self.object {
            Object::Actuator(actuator) => {
                actuator.messages.push(msg_data);
            }
            Object::Substitute(substitute) => {
                substitute.lock().unwrap().messages.push(msg_data);
            }
        }

        self
    }

    pub fn initial(&mut self) -> &mut Self {
        match &mut self.object {
            Object::Actuator(actuator) => {
                actuator.expected_version = Some(Version::initial());
            }
            Object::Substitute(substitute) => {
                substitute.lock().unwrap().expected_version = Some(Version::initial());
            }
        }

        self
    }

    pub async fn execute(&mut self, stream_name: StreamName) -> Result<i64, MessageStoreError> {
        use Object::*;
        match &mut self.object {
            Actuator(actuator) => actuator.execute(stream_name).await,
            Substitute(substitute) => {
                let mut substitute = substitute.lock().unwrap();
                substitute.stream_name = Some(stream_name);

                match substitute.error.take() {
                    Some(err) => Err(err),
                    None => {
                        let position = substitute.messages.len() as i64 - 1;
                        Ok(position)
                    }
                }
            }
        }
    }
}

impl<Settings> HandlerParam<Settings> for Write {
    fn build(connection: Connection, _settings: Settings) -> Self {
        Write::build(connection)
    }
}

#[derive(Clone, Debug)]
pub struct WriteMessages {
    connection: Connection,
    expected_version: Option<Version>,
    messages: Vec<MessageData>,
}

impl WriteMessages {
    pub fn build(connection: Connection) -> Self {
        Self {
            connection,
            expected_version: None,
            messages: Vec::new(),
        }
    }

    #[instrument(name = "Write::execute", skip(self), fields(%stream_name))]
    pub async fn execute(&mut self, stream_name: StreamName) -> Result<i64, MessageStoreError> {
        #[derive(sqlx::FromRow)]
        struct LastPosition(i64);

        let mut last_position = LastPosition(-1);
        let StreamName(ref stream_name) = stream_name;

        let mut transaction = self.connection.begin().await?;

        for message in self.messages.iter() {
            let id = uuid::Uuid::new_v4();
            let version = self.expected_version.as_ref().map(|Version(value)| value);
            let metadata = if message.metadata.is_empty() {
                Value::Null
            } else {
                let map = message.metadata.0.clone();
                Value::Object(map)
            };

            let query = sqlx::query_as(
                "SELECT write_message($1::varchar, $2::varchar, $3::varchar, $4::jsonb, $5::jsonb, $6::bigint);",
            )
                .bind(id.clone())
                .bind(stream_name.clone())
                .bind(&message.type_name)
                .bind(&message.data)
                .bind(metadata)
                .bind(&version);

            trace!(
                "{} [{}, {}, {}, {}, {:?}, {:?}]",
                query.sql(),
                id,
                stream_name,
                message.type_name,
                message.data,
                message.metadata,
                version
            );

            last_position = query.fetch_one(&mut *transaction).await?;

            self.expected_version = self
                .expected_version
                .as_mut()
                .map(|Version(value)| Version(*value + 1));
        }

        transaction.commit().await?;

        let LastPosition(position) = last_position;

        Ok(position)
    }
}

#[derive(Clone, Default, Debug)]
pub struct WriteSubstitute {
    pub error: Option<MessageStoreError>,
    pub expected_version: Option<Version>,
    pub messages: Vec<MessageData>,
    pub stream_name: Option<StreamName>,
}

impl WriteSubstitute {
    pub fn find_message<M>(&self) -> Option<Msg<M>>
    where
        for<'de> M: Message + serde::Deserialize<'de>,
    {
        self.messages
            .iter()
            .find(|message_data| message_data.type_name.as_str() == M::TYPE_NAME)
            .and_then(|message_data| Msg::from_data(message_data.clone()).ok())
    }

    pub fn messages_of_type<M>(&self) -> Vec<Msg<M>>
    where
        for<'de> M: Message + serde::Deserialize<'de>,
    {
        self.messages
            .iter()
            .filter(|message_data| message_data.type_name.as_str() == M::TYPE_NAME)
            .filter_map(|message_data| Msg::from_data(message_data.clone()).ok())
            .collect()
    }
}
