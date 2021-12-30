use std::fmt::{Debug, Display, Formatter};

use cqrs_es::AggregateError;
use persist_es::PersistenceError;
use sqlx::Error;

#[derive(Debug, PartialEq)]
pub enum MysqlAggregateError {
    OptimisticLock,
    ConnectionError(String),
    UnknownError(String),
}

impl Display for MysqlAggregateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MysqlAggregateError::OptimisticLock => write!(f, "optimistic lock error"),
            MysqlAggregateError::UnknownError(msg) => write!(f, "{}", msg),
            MysqlAggregateError::ConnectionError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for MysqlAggregateError {}

impl From<sqlx::Error> for MysqlAggregateError {
    fn from(err: sqlx::Error) -> Self {
        // TODO: improve error handling
        match &err {
            Error::Database(database_error) => {
                if let Some(code) = database_error.code() {
                    if code.as_ref() == "23505" {
                        return MysqlAggregateError::OptimisticLock;
                    }
                }
                MysqlAggregateError::UnknownError(format!("{:?}", err))
            }
            Error::Io(e) => MysqlAggregateError::ConnectionError(e.to_string()),
            Error::Tls(e) => MysqlAggregateError::ConnectionError(e.to_string()),
            Error::Protocol(e) => panic!("sql protocol error encountered: {}", e),
            _ => MysqlAggregateError::UnknownError(format!("{:?}", err)),
        }
    }
}

impl From<MysqlAggregateError> for AggregateError {
    fn from(err: MysqlAggregateError) -> Self {
        match err {
            MysqlAggregateError::OptimisticLock => AggregateError::AggregateConflict,
            MysqlAggregateError::UnknownError(msg) => AggregateError::TechnicalError(msg),
            MysqlAggregateError::ConnectionError(msg) => AggregateError::TechnicalError(msg),
        }
    }
}

impl From<serde_json::Error> for MysqlAggregateError {
    fn from(err: serde_json::Error) -> Self {
        MysqlAggregateError::UnknownError(err.to_string())
    }
}

impl From<MysqlAggregateError> for PersistenceError {
    fn from(err: MysqlAggregateError) -> Self {
        match err {
            MysqlAggregateError::OptimisticLock => PersistenceError::OptimisticLockError,
            MysqlAggregateError::UnknownError(msg) => PersistenceError::UnknownError(msg),
            MysqlAggregateError::ConnectionError(msg) => PersistenceError::ConnectionError(msg),
        }
    }
}
