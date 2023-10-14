use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    WrongExpectedVersion(String),
    #[error("{0}")]
    Database(String),
    #[error("{0}")]
    Pool(String),
    #[error(transparent)]
    Other(SqlxError),
}

impl From<SqlxError> for Error {
    fn from(sqlx_error: SqlxError) -> Self {
        match sqlx_error {
            SqlxError::Database(ref error) => {
                let message = error.message();
                if message.contains("Wrong expected version") {
                    return Self::WrongExpectedVersion(message.to_string());
                }

                Self::Database(format!("{}", sqlx_error))
            }
            SqlxError::PoolClosed | SqlxError::PoolTimedOut => {
                Self::Pool(format!("{}", sqlx_error))
            }
            _ => Self::Other(sqlx_error),
        }
    }
}
