#[cfg(feature = "rock")]
use {
    crate::engine::storage::Flusher,
    parking_lot::RwLock,
    rocksdb::DB,
    std::path::Path,
    std::sync::Arc,
};

use crate::common::operation_error::OperationResult;

pub trait StorageManagerTrait
{
    #[cfg(feature = "rock")]
    fn open<T: AsRef<str>>(
        path: &Path,
        vector_paths: &[T],
    ) -> Result<Arc<RwLock<DB>>, rocksdb::Error>;
    #[cfg(feature = "rock")]
    fn check_db_exists(path: &Path) -> bool;

    #[cfg(feature = "rock")]
    fn new(database: Arc<RwLock<DB>>, column_name: &str) -> Self;

    fn put<K, V>(&self, key: K, value: V) -> OperationResult<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>;

    fn get<K>(&self, key: K) -> OperationResult<Vec<u8>>
    where
        K: AsRef<[u8]>;
    fn remove<K>(&self, key: K) -> OperationResult<()>
    where
        K: AsRef<[u8]>;
    fn flusher(&self) -> Flusher;
}
