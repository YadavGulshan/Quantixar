use std::sync::Arc;

use tokio::{runtime::Runtime, sync::RwLock};

use crate::{common::operation_error::OperationResult, engine::storage::types::StorageConfig};

use super::collections::Collections;

pub struct TableOfContent {
    // collections: Arc<RwLock<Collections>>,
    pub(super) storage_config: Arc<StorageConfig>,
}

impl TableOfContent {
    pub fn new(collections: Arc<RwLock<Collections>>, storage_config: Arc<StorageConfig>) -> Self {
        todo!()
    }

    pub fn insert_vector(
        &self,
        collection_name: &str,
        vector_id: u64,
        vector: Vec<f32>,
    ) -> OperationResult<()> {
        todo!()
    }

    pub fn search_vector(&self, collection_name: &str, vector: Vec<f32>, top: usize) -> Vec<u64> {
        todo!()
    }
}
