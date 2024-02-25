use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use atomic_refcell::AtomicRefCell;

use hnsw_rs::dist::Distance;
use hnsw_rs::hnsw::Hnsw;

use crate::common::operation_error::OperationResult;
use crate::engine::index::hnsw::config::HnswGraphConfig;
use crate::engine::storage::vector::base::VectorStorageEnum;

pub struct HNSWIndex<'b, T: Clone + Send + Sync + 'b, D: Distance<T>> {
  vector_storage: Arc<AtomicRefCell<VectorStorageEnum>>,
  config: HnswGraphConfig,
  path: PathBuf,
  hnsw: Hnsw<'b, T, D>,
}


impl<'b, T: Clone + Send + Sync + 'b, D: Distance<T>> HNSWIndex<'b, T, D> {
  pub fn new(
    vector_storage: Arc<AtomicRefCell<VectorStorageEnum>>,
    config: HnswGraphConfig,
    path: &Path,
  ) -> OperationResult<Self> {
    create_dir_all(path)?;
    let config_path = HnswGraphConfig::get_config_path(path);
    let config = if config_path.exists() {
      HnswGraphConfig::load(&config_path)?
    } else {
      HnswGraphConfig::new(
        config.m,
        config.ef_construct,
        config.full_scan_threshold,
        config.max_indexing_threads,
        config.payload_m,
        config.indexed_vector_count.unwrap(),
      )
    };

    let max_nb_connection = config.m;
    let nb_elem = config.indexed_vector_count.unwrap();
    let nb_layer = 16.min((nb_elem as f32).ln().trunc() as usize);
    let ef_c = config.ef_construct;

    let hnsw = Hnsw::<T, D>::new(max_nb_connection, nb_elem, nb_layer, ef_c, D {});

    Ok(HNSWIndex {
      vector_storage,
      config,
      path: path.to_owned(),
      hnsw,
    })
  }

  fn save_config(&self) -> OperationResult<()> {
    self.config.save(&HnswGraphConfig::get_config_path(&self.path))
  }

  pub fn save(&self) -> OperationResult<()> {
    self.save_config()?;
    Ok(())
  }

  pub fn build_graph(&self) -> OperationResult<()> {
    let vector_storage = self.vector_storage.borrow();

    Ok(())
  }
}
