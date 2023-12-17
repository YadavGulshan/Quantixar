use std::backtrace::Backtrace;
use std::io::{Error as IoError, ErrorKind};
use thiserror::Error;
use crate::utils::mem::Mem;

pub type OperationResult<T> = Result<T, OperationError>;

#[derive(Error, Debug, Clone)]
#[error("{0}")]
pub enum OperationError {
    /// Service Error prevents further update of the collection until it is fixed.
    /// Should only be used for hardware, data corruption, IO, or other unexpected internal errors.
    #[error("Service runtime error: {description}")]
    ServiceError {
        description: String,
        backtrace: Option<String>,
    },
    #[error("Inconsistent storage: {description}")]
    InconsistentStorage { description: String },
    #[error("Out of memory, free: {free}, {description}")]
    OutOfMemory { description: String, free: u64 },
    #[error("Operation cancelled: {description}")]
    Cancelled { description: String },
    #[error("Validation failed: {description}")]
    ValidationError { description: String },
    #[error("Wrong usage of sparse vectors")]
    WrongSparse,
}

impl OperationError {
    pub fn service_error(description: impl Into<String>) -> OperationError {
        OperationError::ServiceError {
            description: description.into(),
            backtrace: Some(Backtrace::force_capture().to_string()),
        }
    }
}

impl From<IoError> for OperationError {
    fn from(err: IoError) -> Self {
        match err.kind() {
            ErrorKind::OutOfMemory => {
                let free_memory = Mem::new().available_memory_bytes();
                OperationError::OutOfMemory {
                    description: format!("IO Error: {err}"),
                    free: free_memory,
                }
            }
            _ => OperationError::service_error(format!("IO Error: {err}")),
        }
    }
}

pub fn get_service_error<T>(err: &OperationResult<T>) -> Option<OperationError> {
    match err {
        Ok(_) => None,
        Err(error) => match error {
            OperationError::ServiceError { .. } => Some(error.clone()),
            _ => None,
        },
    }
}



