use std::{collections::HashMap, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::{runtime::Handle, sync::RwLock};
use validator::Validate;

pub type CollectionId = String;

pub type Collections = HashMap<CollectionId, Collection>;

pub struct Collection {
    pub(super) id: CollectionId,
    pub(crate) collection_config: Arc<RwLock<CollectionConfig>>,
    update_runtime: Handle,
    // Search runtime handle.
    search_runtime: Handle,
    updates_lock: RwLock<()>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Validate, Clone, PartialEq, Eq)]
pub struct CollectionConfig {
    #[validate]
    pub hnsw_config: HnswConfig,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Validate, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct HnswConfig {
    /// Number of edges per node in the index graph. Larger the value - more accurate the search, more space required.
    pub m: usize,
    /// Number of neighbours to consider during the index building. Larger the value - more accurate the search, more time required to build index.
    #[validate(range(min = 4))]
    pub ef_construct: usize,
    /// Minimal size (in KiloBytes) of vectors for additional payload-based indexing.
    /// If payload chunk is smaller than `full_scan_threshold_kb` additional indexing won't be used -
    /// in this case full-scan search should be preferred by query planner and additional indexing is not required.
    /// Note: 1Kb = 1 vector of size 256
    #[serde(alias = "full_scan_threshold_kb")]
    pub full_scan_threshold: usize,
    /// Number of parallel threads used for background index building. If 0 - auto selection.
    #[serde(default = "default_max_indexing_threads")]
    pub max_indexing_threads: usize,
    /// Store HNSW index on disk. If set to false, index will be stored in RAM. Default: false
    #[serde(default, skip_serializing_if = "Option::is_none")] // Better backward compatibility
    pub on_disk: Option<bool>,
    /// Custom M param for hnsw graph built for payload index. If not set, default M will be used.
    #[serde(default, skip_serializing_if = "Option::is_none")] // Better backward compatibility
    pub payload_m: Option<usize>,
}

const fn default_max_indexing_threads() -> usize {
    0
}
