use std::{backtrace::Backtrace, collections::TryReserveError, io::{
  Error as IoError,
  ErrorKind,
}, sync::atomic::{
  AtomicBool,
  Ordering,
}};
use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use atomicwrites::Error as AtomicIoError;
use hdf5::Error;
use rayon::ThreadPoolBuildError;
use thiserror::Error;

use crate::{
  common::{
    point_id::PointIdType,
    types::{
      PayloadKeyType,
      SeqNumberType,
    },
  },
  utils::mem::Mem,
};

pub const PROCESS_CANCELLED_BY_SERVICE_MESSAGE: &str = "process cancelled by service";

#[derive(Error, Debug, Clone)]
#[error("{0}")]
pub enum OperationError
{
  #[error("Vector inserting error: expected dim: {expected_dim}, got {received_dim}")]
  WrongVector
  {
    expected_dim: usize,
    received_dim: usize,
  },
  #[error("Not existing vector name error: {received_name}")]
  VectorNameNotExists
  {
    received_name: String
  },
  #[error("Missed vector name error: {received_name}")]
  MissedVectorName
  {
    received_name: String
  },
  #[error("No point with id {missed_point_id}")]
  PointIdError
  {
    missed_point_id: PointIdType
  },
  #[error("Payload type does not match with previously given for field {field_name}. Expected: {expected_type}")]
  TypeError
  {
    field_name: PayloadKeyType,
    expected_type: String,
  },
  #[error("Unable to infer type for the field '{field_name}'. Please specify `field_type`")]
  TypeInferenceError
  {
    field_name: PayloadKeyType
  },
  /// Service Error prevents further update of the collection until it is fixed.
  /// Should only be used for hardware, data corruption, IO, or other unexpected internal errors.
  #[error("Service runtime error: {description}")]
  ServiceError
  {
    description: String,
    backtrace: Option<String>,
  },
  #[error("Inconsistent storage: {description}")]
  InconsistentStorage
  {
    description: String
  },
  #[error("Out of memory, free: {free}, {description}")]
  OutOfMemory
  {
    description: String,
    free: u64,
  },
  #[error("Operation cancelled: {description}")]
  Cancelled
  {
    description: String
  },
  #[error("Validation failed: {description}")]
  ValidationError
  {
    description: String
  },
  #[error("Wrong usage of sparse vectors")]
  WrongSparse,
}

impl OperationError
{
  pub fn service_error(description: impl Into<String>) -> OperationError
  {
    OperationError::ServiceError {
      description: description.into(),
      backtrace: Some(Backtrace::force_capture().to_string()),
    }
  }
}

pub fn check_process_stopped(stopped: &AtomicBool) -> OperationResult<()>
{
  if stopped.load(Ordering::Relaxed) {
    return Err(OperationError::Cancelled {
      description: PROCESS_CANCELLED_BY_SERVICE_MESSAGE.to_string(),
    });
  }
  Ok(())
}

/// Contains information regarding last operation error, which should be fixed before next operation
/// could be processed
#[derive(Debug, Clone)]
pub struct SegmentFailedState
{
  pub version: SeqNumberType,
  pub point_id: Option<PointIdType>,
  pub error: OperationError,
}

impl From<ThreadPoolBuildError> for OperationError
{
  fn from(error: ThreadPoolBuildError) -> Self
  {
    OperationError::ServiceError {
      description: format!("{error}"),
      backtrace: Some(Backtrace::force_capture().to_string()),
    }
  }
}

impl From<serde_cbor::Error> for OperationError
{
  fn from(err: serde_cbor::Error) -> Self
  {
    OperationError::service_error(format!("Failed to parse data: {err}"))
  }
}

impl<E> From<AtomicIoError<E>> for OperationError
{
  fn from(err: AtomicIoError<E>) -> Self
  {
    match err {
      AtomicIoError::Internal(io_err) => OperationError::from(io_err),
      AtomicIoError::User(_user_err) => {
        OperationError::service_error("Unknown atomic write error")
      }
    }
  }
}

impl From<IoError> for OperationError
{
  fn from(err: IoError) -> Self
  {
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

impl From<serde_json::Error> for OperationError
{
  fn from(err: serde_json::Error) -> Self
  {
    OperationError::service_error(format!("Json error: {err}"))
  }
}

impl From<fs_extra::error::Error> for OperationError
{
  fn from(err: fs_extra::error::Error) -> Self
  {
    OperationError::service_error(format!("File system error: {err}"))
  }
}


impl From<TryReserveError> for OperationError
{
  fn from(err: TryReserveError) -> Self
  {
    let free_memory = Mem::new().available_memory_bytes();
    OperationError::OutOfMemory {
      description: format!("Failed to reserve memory: {err}"),
      free: free_memory,
    }
  }
}

impl From<hdf5::Error> for OperationError {
  fn from(value: Error) -> Self {
    OperationError::ServiceError {
      description: format!("HDF5 error: {value}"),
      backtrace: Some(Backtrace::force_capture().to_string()),
    }
  }
}

impl ResponseError for OperationError {
  fn status_code(&self) -> StatusCode {
    match *self {
      OperationError::WrongVector { .. } => StatusCode::BAD_REQUEST,
      OperationError::VectorNameNotExists { .. } => StatusCode::NOT_FOUND,
      OperationError::MissedVectorName { .. } => StatusCode::BAD_REQUEST,
      OperationError::PointIdError { .. } => StatusCode::NOT_FOUND,
      OperationError::TypeError { .. } => StatusCode::BAD_REQUEST,
      OperationError::TypeInferenceError { .. } => StatusCode::BAD_REQUEST,
      OperationError::ServiceError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
      OperationError::InconsistentStorage { .. } => StatusCode::INTERNAL_SERVER_ERROR,
      OperationError::OutOfMemory { .. } => StatusCode::INSUFFICIENT_STORAGE,
      OperationError::Cancelled { .. } => StatusCode::SERVICE_UNAVAILABLE,
      OperationError::ValidationError { .. } => StatusCode::BAD_REQUEST,
      OperationError::WrongSparse => StatusCode::BAD_REQUEST,
    }
  }
  fn error_response(&self) -> HttpResponse {
    HttpResponse::build(self.status_code())
            .content_type("application/json")
            .body(self.to_string())
  }
}


pub type OperationResult<T> = Result<T, OperationError>;
