use std::sync::Arc;

use tokio::{runtime::Runtime, sync::RwLock};

use crate::engine::storage::types::StorageConfig;
use crate::engine::table::collections::Collections;

pub struct TableOfContent {
  collections: Arc<RwLock<Collections>>,
  pub(super) storage_config: Arc<StorageConfig>,
  search_runtime: Runtime,
  update_runtime: Runtime,
  general_runtime: Runtime,
}
