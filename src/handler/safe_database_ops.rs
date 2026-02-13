use remdb::RemDb;
use remdb::transaction::{IsolationLevel, TransactionType};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DatabaseError {
    TransactionError(String),
    CommitError(String),
    RollbackError(String),
    QueryError(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            DatabaseError::CommitError(msg) => write!(f, "Commit error: {}", msg),
            DatabaseError::RollbackError(msg) => write!(f, "Rollback error: {}", msg),
            DatabaseError::QueryError(msg) => write!(f, "Query error: {}", msg),
        }
    }
}

impl Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub struct SafeDatabaseOperations;

impl SafeDatabaseOperations {
    pub fn begin_transaction(
        db: &mut RemDb,
        tx_type: TransactionType,
        isolation_level: IsolationLevel,
    ) -> Result<(), DatabaseError> {
        unsafe {
            match db.begin_transaction(
                tx_type,
                isolation_level,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            ) {
                Ok(_) => Ok(()),
                Err(err) => Err(DatabaseError::TransactionError(format!("{:?}", err))),
            }
        }
    }

    pub fn commit_transaction(db: &mut RemDb) -> Result<(), DatabaseError> {
        unsafe {
            match db.commit_transaction() {
                Ok(_) => Ok(()),
                Err(err) => Err(DatabaseError::CommitError(format!("{:?}", err))),
            }
        }
    }

    pub fn rollback_transaction(db: &mut RemDb) -> Result<(), DatabaseError> {
        unsafe {
            match db.rollback_transaction() {
                Ok(_) => Ok(()),
                Err(err) => Err(DatabaseError::RollbackError(format!("{:?}", err))),
            }
        }
    }

    pub fn flush_logs(db: &mut RemDb) -> Result<(), DatabaseError> {
        unsafe {
            match db.flush_logs() {
                Ok(_) => Ok(()),
                Err(err) => Err(DatabaseError::QueryError(format!(
                    "Flush logs error: {:?}",
                    err
                ))),
            }
        }
    }
}
