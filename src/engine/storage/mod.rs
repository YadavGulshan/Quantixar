#[cfg(feature = "rock")]
use rocksdb::{DBRecoveryMode, LogLevel, Options};

use crate::common::operation_error::OperationResult;
#[cfg(feature = "rock")]
mod rocksdb_buffered_delete_wrapper;
#[cfg(feature = "rock")]
mod rocksdb_wrapper;
#[cfg(feature = "rock")]
mod storage_manager;
#[cfg(feature = "rock")]
mod storage_mgr_opts;
pub mod types;
#[cfg(feature = "rock")]
pub type Flusher = Box<dyn FnOnce() -> OperationResult<()> + Send>;

const DB_CACHE_SIZE: usize = 10 * 1024 * 1024;
// 10 mb
const DB_MAX_LOG_SIZE: usize = 1024 * 1024;
// 1 mb
const DB_MAX_OPEN_FILES: usize = 256;
const DB_DELETE_OBSOLETE_FILES_PERIOD: u64 = 3 * 60 * 1_000_000; // 3 minutes in microseconds

pub const DB_VECTOR_CF: &str = "vector";
pub const DB_PAYLOAD_CF: &str = "payload";
pub const DB_MAPPING_CF: &str = "mapping";
pub const DB_VERSIONS_CF: &str = "version";

#[cfg(feature = "rock")]
pub fn db_options() -> Options {
    let mut options: Options = Options::default();
    options.set_write_buffer_size(DB_CACHE_SIZE);
    options.create_if_missing(true);
    options.set_log_level(LogLevel::Error);
    options.set_recycle_log_file_num(1);
    options.set_keep_log_file_num(1); // must be greater than zero
    options.set_max_log_file_size(DB_MAX_LOG_SIZE);
    options.set_delete_obsolete_files_period_micros(DB_DELETE_OBSOLETE_FILES_PERIOD);
    options.create_missing_column_families(true);
    options.set_max_open_files(DB_MAX_OPEN_FILES as i32);

    // Qdrant relies on it's own WAL for durability
    options.set_wal_recovery_mode(DBRecoveryMode::TolerateCorruptedTailRecords);
    #[cfg(debug_assertions)]
    {
        options.set_paranoid_checks(true);
    }
    options
}
