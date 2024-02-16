pub mod collections;

use std::sync::Arc;

use tokio::{runtime::Runtime, sync::RwLock};

use self::collections::Collections;

use super::storage::types::StorageConfig;

pub struct TableOfContent {
    collections: Arc<RwLock<Collections>>,
    pub(super) storage_config: Arc<StorageConfig>,
    search_runtime: Runtime,
    update_runtime: Runtime,
    general_runtime: Runtime,
}
