use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use io::file_operations::{atomic_save_json, read_json};

use crate::common::operation_error::OperationResult;

pub const HNSW_INDEX_CONFIG_FILE: &str = "hnsw_config.json";

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
pub struct HnswGraphConfig {
  pub m: usize,
  pub ef_construct: usize,
  /// Number of neighbours to search on construction
  pub ef: usize,
  #[serde(default)]
  pub max_nb_connection: usize,
  #[serde(default)]
  pub keep_pruned: bool,
  #[serde(default)]
  pub max_layer: usize,
  #[serde(default)]
  pub data_dimension: usize,

  /// Maximum number of neighbours to consider when searching
  #[serde(default)]
  pub knbn_max: usize,

  /// Number of neighbours to consider when searching
  #[serde(default)]
  pub knbn: usize,
  #[serde(default)]
  pub searching: bool,
  #[serde(default)]
  pub extend_candidates: bool,
  #[serde(default)]
  pub dataset_size: usize,
}

impl HnswGraphConfig {
  pub fn new(
    m: usize,
    ef_construct: usize,
    ef: usize,
    max_nb_connection: usize,
    keep_pruned: bool,
    max_layer: usize,
    data_dimension: usize,
    searching: bool,
    extend_candidates: bool,
    data_set_size: usize,
    knbn_max: usize,
    knbn: usize
  ) -> Self {
    HnswGraphConfig {
      m,
      ef_construct,
      ef,
      max_nb_connection,
      keep_pruned,
      max_layer,
      data_dimension,
      searching,
      extend_candidates,
      dataset_size: data_set_size,
      knbn_max,
      knbn
    }
  }


  pub fn get_config_path(path: &Path) -> PathBuf {
    path.join(HNSW_INDEX_CONFIG_FILE)
  }

  pub fn load(path: &Path) -> OperationResult<Self> {
    Ok(read_json(path)?)
  }

  pub fn save(&self, path: &Path) -> OperationResult<()> {
    Ok(atomic_save_json(path, self)?)
  }
}
