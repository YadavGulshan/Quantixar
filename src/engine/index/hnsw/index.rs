use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use atomic_refcell::AtomicRefCell;

use clap::Id;
use hnsw_rs::{
    api::AnnT,
    dist::{DistCosine, DistL2, Distance},
    hnsw::{self, Hnsw, Neighbour},
    hnswio::HnswIo,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::{Map, Value};

use crate::{
    actix::{handlers::vector, model::vector::AddVector},
    common::operation_error::OperationError,
    engine::{
        storage::vector::base::VectorStorage,
        types::{
            types::{Payload, PointOffsetType, VectorElementType},
            vector::VectorRef,
        },
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
    hnsw: Hnsw<'b, f32, DistCosine>,
}

impl<'b> HNSWIndex<'b> {
    pub fn new(
        vector_storage: Arc<AtomicRefCell<VectorStorageEnum>>,
        path: &Path,
        data_dimension: usize,
        dataset_size: usize,
        dist_f: DistCosine,
    ) -> OperationResult<Self> {
        create_dir_all(path)?;
        let config_path = HnswGraphConfig::get_config_path(path);
        let config = if config_path.exists() {
            HnswGraphConfig::load(&config_path)?
        } else {
            HnswGraphConfig::new(
                1000,
                400,
                24,
                15,
                false,
                16,
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
        let hnsw = Hnsw::<f32, DistCosine>::new(
            config.max_nb_connection,
            config.m,
            config.max_layer,
            config.ef_construct,
            dist_f,
        );

        let hnsw_index = HNSWIndex {
            vector_storage,
            config,
            path: path.to_path_buf(),
            hnsw,
        };

        Ok(hnsw_index)
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
        log::info!("Building HNSW graph");
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
            self.hnsw.parallel_insert_slice(&data_for_insertion);
        } else {
            for d in data_for_insertion {
                self.hnsw.insert_slice(d);
            }
        }
        Ok(())
    }

    pub fn add(&mut self, vector: &[VectorElementType], payload: Payload) -> OperationResult<()> {
        let key = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        let vector_ref = VectorRef::Dense(vector);
        let mut vector_storage = self.vector_storage.borrow_mut();

        match vector_storage.insert_vector(key as PointOffsetType, vector_ref, payload) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        };

        let data_with_id: (&[VectorElementType], usize) = (vector, key);
        self.hnsw.insert_slice(data_with_id);
        Ok(())
    }

    pub fn batch_add(&mut self, vectors: Vec<AddVector>) -> OperationResult<()> {
        let mut vector_storage = self.vector_storage.borrow_mut();
        let mut data = Vec::new();
        for vector in vectors {
            let key: usize = vector_storage.total_vector_count() + 1;
            let vector_ref = VectorRef::Dense(vector.vectors.as_slice());
            match vector_storage.insert_vector(key as PointOffsetType, vector_ref, vector.payload) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            };

            data.push((vector.vectors.clone(), key));
        }
        let datas_with_id = data
            .par_iter()
            .map(|x| (x.0.as_slice(), x.1))
            .collect::<Vec<(&[VectorElementType], usize)>>();
        self.hnsw.parallel_insert_slice(&datas_with_id);
        Ok(())
    }

    pub fn search(&self, query: &[f32], k: usize) -> OperationResult<Vec<Map<String, Value>>> {
        let neighbours: Vec<Neighbour> = self.hnsw.search(&query, k, self.config.ef_construct);

        let payloads = neighbours
            .iter()
            .map(|x| {
                let payload = self
                    .vector_storage
                    .borrow()
                    .get_payload(x.d_id as u32)
                    .unwrap();
                let mut map = Map::new();
                map.insert(
                    "id".to_string(),
                    Value::Number(serde_json::Number::from(x.d_id as u32)),
                );
                map.insert("payload".to_string(), Value::Object(payload.0));
                map.insert(
                    "distance".to_string(),
                    Value::Number(serde_json::Number::from_f64(x.distance as f64).unwrap()),
                );
                map
            })
            .collect::<Vec<Map<String, Value>>>();
        Ok(payloads)
    }

    pub fn dump(&self) -> OperationResult<()> {
        self.hnsw.dump_layer_info();
        let filename = String::from("dumpreloadgraph");
        match self.hnsw.file_dump(&filename) {
            Ok(_) => {
                let vector_storage = self.vector_storage.borrow();
                vector_storage.get_dense_storage().save_payloads()?;
                Ok(())
            }
            Err(e) => {
                log::error!("Error dumping graph: {}", e);
                Err(OperationError::InternalError)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::engine::storage::vector::dense_vector_storage::SimpleDenseVectorStorage;
    use crate::engine::types::distance::Distance;


}
