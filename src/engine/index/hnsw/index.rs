use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::Arc,
};

use atomic_refcell::AtomicRefCell;

use hnsw_rs::{dist::Distance, hnsw::Hnsw};

use crate::engine::storage::vector::base::VectorStorage;
use crate::{
    common::operation_error::OperationResult,
    engine::{index::hnsw::config::HnswGraphConfig, storage::vector::base::VectorStorageEnum},
};

pub struct HNSWIndex<'b, T: Clone + Send + Sync + 'b, D: Distance<T> + Send + Sync> {
    vector_storage: Arc<AtomicRefCell<VectorStorageEnum>>,
    config: HnswGraphConfig,
    path: PathBuf,
    hnsw: Hnsw<'b, T, D>,
}

impl<'b, T: Clone + Send + Sync + 'b, D: Distance<T> + Send + Sync> HNSWIndex<'b, T, D> {
    pub fn new(
        vector_storage: Arc<AtomicRefCell<VectorStorageEnum>>,
        path: &Path,
        data_dimension: usize,
        dataset_size: usize,
        dist_f: D,
    ) -> OperationResult<Self> {
        create_dir_all(path)?;
        let config_path = HnswGraphConfig::get_config_path(path);
        let config = if config_path.exists() {
            HnswGraphConfig::load(&config_path)?
        } else {
            HnswGraphConfig::new(
                24,
                400,
                24,
                0,
                false,
                10,
                data_dimension,
                false,
                false,
                dataset_size,
            )
        };

        let max_nb_connection = config.m;
        let nb_elem = config.dataset_size;
        let nb_layer = 16.min((nb_elem as f32).ln().trunc() as usize);
        let ef_c = config.ef_construct;

        let hnsw = Hnsw::<T, D>::new(
            config.max_nb_connection,
            config.m,
            config.max_layer,
            config.ef_construct,
            dist_f,
        );

        Ok(HNSWIndex {
            vector_storage,
            config,
            path: path.to_owned(),
            hnsw,
        })
    }

    fn save_config(&self) -> OperationResult<()> {
        self.config
            .save(&HnswGraphConfig::get_config_path(&self.path))
    }

    pub fn save(&self) -> OperationResult<()> {
        self.save_config()?;
        Ok(())
    }

    pub fn build_graph(&mut self, parallel_insertion: bool) -> OperationResult<()> {
        let vector_storage = self.vector_storage.borrow();
        self.hnsw
            .set_extend_candidates(self.config.extend_candidates);
        let vector_storage = self.vector_storage.borrow();
        let total_vector_count = vector_storage.total_vector_count();
        let deleted_bitslice = vector_storage.deleted_vector_bitslice();

        let data_for_insertion: Vec<(&[f32], usize)> = vector_storage
            .get_dense_storage()
            .vectors()
            .get_all_vectors();

        if parallel_insertion {
            self.hnsw.parallel_insert_slice(&data_for_insertion);
        }

        Ok(())
    }
}
