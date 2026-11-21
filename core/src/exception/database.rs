use crate::constants::ErrorCode;
use crate::exception::base::{AnyExceptionObject, BaseException};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DatabaseException {
    #[error("SQLite database error: {0}")]
    Sqlite(String),

    #[error("Record not found for id: {0}")]
    NotFound(i64),

    #[error("Schema migration or initialization failed: {0}")]
    MigrationFailed(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Database lock poisoned or busy: {0}")]
    LockError(String),

    #[error("Database exception [{code} - {value}]: {message}")]
    Custom {
        code: i32,
        value: &'static str,
        message: String,
        object: Option<AnyExceptionObject>,
    },
}

impl DatabaseException {
    pub fn custom<T: 'static + Send + Sync>(
        code: i32,
        value: &'static str,
        message: impl Into<String>,
        object: Option<T>,
    ) -> Self {
        Self::Custom {
            code,
            value,
            message: message.into(),
            object: object.map(AnyExceptionObject::new),
        }
    }
}

impl BaseException for DatabaseException {
    fn code(&self) -> i32 {
        match self {
            Self::Sqlite(_) => ErrorCode::DatabaseError.code(),
            Self::NotFound(_) => ErrorCode::RecordNotFound.code(),
            Self::MigrationFailed(_) => ErrorCode::DatabaseMigrationFailed.code(),
            Self::ConstraintViolation(_) => ErrorCode::ConstraintViolation.code(),
            Self::LockError(_) => ErrorCode::DatabaseLockError.code(),
            Self::Custom { code, .. } => *code,
        }
    }

    fn value(&self) -> &str {
        match self {
            Self::Sqlite(_) => ErrorCode::DatabaseError.value(),
            Self::NotFound(_) => ErrorCode::RecordNotFound.value(),
            Self::MigrationFailed(_) => ErrorCode::DatabaseMigrationFailed.value(),
            Self::ConstraintViolation(_) => ErrorCode::ConstraintViolation.value(),
            Self::LockError(_) => ErrorCode::DatabaseLockError.value(),
            Self::Custom { value, .. } => value,
        }
    }

    fn object(&self) -> Option<&(dyn std::any::Any + Send + Sync)> {
        match self {
            Self::Sqlite(msg) => Some(msg),
            Self::NotFound(id) => Some(id),
            Self::MigrationFailed(msg) => Some(msg),
            Self::ConstraintViolation(msg) => Some(msg),
            Self::LockError(msg) => Some(msg),
            Self::Custom { object, .. } => object.as_ref().map(|o| &*o.0 as &(dyn std::any::Any + Send + Sync)),
        }
    }
}
