use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("{0}")]
    WrongExpectedVersion(String),
    #[error("{0}")]
    Database(String),
    #[error("{0}")]
    Pool(String),
    #[error("{0}")]
    Other(String),
}

impl From<SqlxError> for Error {
    fn from(sqlx_error: SqlxError) -> Self {
        match sqlx_error {
            SqlxError::Database(ref error) => {
                let message = error.message();
                if message.contains("Wrong expected version") {
                    return Self::WrongExpectedVersion(message.to_string());
                }

                Self::Database(sqlx_error.to_string())
            }
            SqlxError::PoolClosed | SqlxError::PoolTimedOut => Self::Pool(sqlx_error.to_string()),
            _ => Self::Other(sqlx_error.to_string()),
        }
    }
}
