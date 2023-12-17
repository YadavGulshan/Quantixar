use crate::common::operation_error::OperationResult;

pub mod operation_error;
pub mod mmap_type;

pub type Flusher = Box<dyn FnOnce() -> OperationResult<()> + Send>;
