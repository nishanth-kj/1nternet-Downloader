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
}
