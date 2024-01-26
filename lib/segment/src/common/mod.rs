use crate::common::operation_error::OperationResult;

pub mod mmap_type;
pub mod operation_error;

pub type Flusher = Box<dyn FnOnce() -> OperationResult<()> + Send>;
