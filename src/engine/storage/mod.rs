use crate::common::operation_error::OperationResult;

mod storage_mgr;
mod storage_mgr_trait;
mod storage_mgr_opts;


pub type Flusher = Box<dyn FnOnce() -> OperationResult<()> + Send>;
