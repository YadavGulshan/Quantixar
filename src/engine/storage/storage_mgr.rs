#[cfg(feature = "rock")]
extern crate rocksdb;

#[cfg(feature = "rock")]
use rocksdb::{BlockBasedOptions, DB};

use crate::engine::storage::storage_mgr_opts::StorageManagerOptions;
use crate::engine::storage::storage_mgr_trait::StorageManagerTrait;

pub struct StorageManager {
  root_path: String,
  name: String,
  size: usize,
  options: StorageManagerOptions,
  #[cfg(feature = "rock")]
  db: Arc<RwLock<DB>>,
  #[cfg(feature = "rock")]
  table_options: BlockBasedOptions,

  #[cfg(feature = "etcd")]
  db: String,
  #[cfg(feature = "etcd")]
  table_options: String,
}


impl StorageManagerTrait for StorageManager {}
