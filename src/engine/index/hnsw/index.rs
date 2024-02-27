use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use atomic_refcell::AtomicRefCell;

use hnsw_rs::{
    dist::{DistL2, Distance},
    hnsw::{Hnsw, Neighbour},
};

use crate::{
    actix::handlers::vector,
    engine::{
        storage::vector::base::VectorStorage,
        types::{types::VectorElementType, vector::VectorRef},
    },
};
use crate::{
    common::operation_error::OperationResult,
    engine::{index::hnsw::config::HnswGraphConfig, storage::vector::base::VectorStorageEnum},
};

#[derive(Clone)]
pub struct HNSWIndex<'b> {
    vector_storage: Arc<AtomicRefCell<VectorStorageEnum>>,
    config: HnswGraphConfig,
    path: PathBuf,
    hnsw: Hnsw<'b, f32, DistL2>,
}

impl<'b> HNSWIndex<'b> {
    pub fn new(
        vector_storage: Arc<AtomicRefCell<VectorStorageEnum>>,
        path: &Path,
        data_dimension: usize,
        dataset_size: usize,
        dist_f: DistL2,
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
                100,
                10,
            )
        };

        let max_nb_connection = config.m;
        let nb_elem = config.dataset_size;
        let nb_layer = 16.min((nb_elem as f32).ln().trunc() as usize);
        let ef_c = config.ef_construct;

        let hnsw = Hnsw::<f32, DistL2>::new(
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

        let data_for_insertion = vector_storage
            .get_dense_storage()
            .vectors()
            .get_all_vectors();

        dbg!(total_vector_count);
        if parallel_insertion {
            log::info!("Performing parallel insertion");
            self.hnsw.parallel_insert_slice(&data_for_insertion);
        } else {
            log::info!("Performing serial insertion");
            for d in data_for_insertion {
                self.hnsw.insert_slice(d);
            }
        }

        Ok(())
    }

    pub fn add(&mut self, vector: &[VectorElementType]) -> OperationResult<()> {
        log::info!("Adding vector to hnsw index");
        let key = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        let vector_ref = VectorRef::Dense(vector);
        match self
            .vector_storage
            .borrow_mut()
            .insert_vector(key as u32, vector_ref)
        {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        };
        let data_with_id: (&[VectorElementType], usize) = (vector, key);
        self.hnsw.insert_slice(data_with_id);
        Ok(())
    }

    pub fn search(&self, query: &[f32], k: usize) -> OperationResult<Vec<usize>> {
        let neighbours: Vec<Neighbour> = self.hnsw.search(&query, k, self.config.ef_construct);
        let result: Vec<usize> = neighbours.iter().map(|x| x.d_id).collect();
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::engine::storage::vector::dense_vector_storage::SimpleDenseVectorStorage;
    use crate::engine::types::distance::Distance;

    #[test]
    fn test_hnsw_index() {
        let dim = 3;

        let coloumn_name = "test";
        use hnsw_rs::dist::DistL2;

        // Assuming `dim` is the dimension of your vectors and `path` is a valid path
        let vector_storage = Arc::new(AtomicRefCell::new(VectorStorageEnum::DenseSimple(
            SimpleDenseVectorStorage::new(dim, Distance::Euclidean, "test"),
        )));
        let path = Path::new("test");
        let mut hnsw_index = HNSWIndex::new(vector_storage, path, dim, 10, DistL2).unwrap();
        hnsw_index.build_graph(false).unwrap();
        hnsw_index.save_config().unwrap();

        let query = vec![0.0, 0.0, 0.0];
        let k = 3;
        let result = hnsw_index.search(&query, k).unwrap();
        println!("{:?}", result);
    }
}
