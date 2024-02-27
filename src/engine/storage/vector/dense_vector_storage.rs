use std::mem::size_of;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use actix_web::dev::Path;
use atomic_refcell::AtomicRefCell;
use bitvec::prelude::{BitSlice, BitVec};
use log::debug;
use parking_lot::RwLock;
use rocksdb::DB;
use serde::{Deserialize, Serialize};

use crate::common::operation_error::{check_process_stopped, OperationError, OperationResult};
use crate::engine::storage::rocksdb::rocksdb_wrapper::DatabaseColumnWrapper;
use crate::engine::storage::rocksdb::storage_manager::StorageManager;
use crate::engine::storage::rocksdb::Flusher;
use crate::engine::storage::vector::base::{DenseVectorStorage, VectorStorage, VectorStorageEnum};
use crate::engine::storage::vector::bitvec::bitvec_set_deleted;
use crate::engine::storage::vector::chunked_vectors::ChunkedVectors;
use crate::engine::types::cow_vector::CowVector;
use crate::engine::types::distance::Distance;
use crate::engine::types::types::{PointOffsetType, VectorElementType};
use crate::engine::types::vector::VectorRef;

/// In-memory vector storage with on-update persistence using `store`
pub struct SimpleDenseVectorStorage {
    dim: usize,
    distance: Distance,
    vectors: ChunkedVectors<VectorElementType>,
    db_wrapper: DatabaseColumnWrapper,
    update_buffer: StoredRecord,
    /// BitVec for deleted flags. Grows dynamically upto last set flag.
    deleted: BitVec,
    /// Current number of deleted vectors.
    deleted_count: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct StoredRecord {
    pub deleted: bool,
    pub vector: Vec<VectorElementType>,
}

pub fn open_simple_vector_storage(
    database: Arc<RwLock<DB>>,
    database_column_name: &str,
    dim: usize,
    distance: Distance,
) -> OperationResult<Arc<AtomicRefCell<VectorStorageEnum>>> {
    let mut vectors = ChunkedVectors::new(dim);
    let (mut deleted, mut deleted_count) = (BitVec::new(), 0);

    let db_wrapper = DatabaseColumnWrapper::new(database, database_column_name);

    for (key, value) in db_wrapper.lock_db().iter()? {
        let point_id: PointOffsetType = bincode::deserialize(&key)
            .map_err(|_| OperationError::service_error("cannot deserialize point id from db"))?;
        let stored_record: StoredRecord = bincode::deserialize(&value)
            .map_err(|_| OperationError::service_error("cannot deserialize record from db"))?;

        // Propagate deleted flag
        if stored_record.deleted {
            bitvec_set_deleted(&mut deleted, point_id, true);
            deleted_count += 1;
        }
        vectors.insert(point_id, &stored_record.vector)?;
    }

    debug!("Segment vectors: {}", vectors.len());
    debug!(
        "Estimated segment size {} MB",
        vectors.len() * dim * size_of::<VectorElementType>() / 1024 / 1024
    );

    Ok(Arc::new(AtomicRefCell::new(
        VectorStorageEnum::DenseSimple(SimpleDenseVectorStorage {
            dim,
            distance,
            vectors,
            db_wrapper,
            update_buffer: StoredRecord {
                deleted: false,
                vector: vec![0.; dim],
            },
            deleted,
            deleted_count,
        }),
    )))
}

impl SimpleDenseVectorStorage {
    pub fn new(dim: usize, distance: Distance, coloumn_name: &str) -> SimpleDenseVectorStorage {
        let vectors = ChunkedVectors::new(dim);
        let db = StorageManager::open_db_with_existing_cf(&std::path::Path::new(coloumn_name)).unwrap();
        let db_wrapper = DatabaseColumnWrapper::new(
            db,
            coloumn_name,
        );
        db_wrapper.create_column_family_if_not_exists().unwrap();
        SimpleDenseVectorStorage {
            dim,
            distance,
            vectors,
            db_wrapper,
            update_buffer: StoredRecord {
                deleted: false,
                vector: vec![0.; dim],
            },
            deleted: BitVec::new(),
            deleted_count: 0,
        }
    }

    /// Set deleted flag for given key. Returns previous deleted state.
    #[inline]
    fn set_deleted(&mut self, key: PointOffsetType, deleted: bool) -> bool {
        if key as usize >= self.vectors.len() {
            return false;
        }
        let was_deleted = bitvec_set_deleted(&mut self.deleted, key, deleted);
        if was_deleted != deleted {
            if !was_deleted {
                self.deleted_count += 1;
            } else {
                self.deleted_count -= 1;
            }
        }
        was_deleted
    }

    pub fn vectors(&self) -> &ChunkedVectors<VectorElementType> {
        &self.vectors
    }

    fn update_stored(
        &mut self,
        key: PointOffsetType,
        deleted: bool,
        vector: Option<&[VectorElementType]>,
    ) -> OperationResult<()> {
        // Write vector state to buffer record
        let record = &mut self.update_buffer;
        record.deleted = deleted;
        if let Some(vector) = vector {
            record.vector.copy_from_slice(vector);
        }

        // Store updated record
        self.db_wrapper.put(
            bincode::serialize(&key).unwrap(),
            bincode::serialize(&record).unwrap(),
        )?;

        Ok(())
    }
}

impl DenseVectorStorage for SimpleDenseVectorStorage {
    fn get_dense(&self, key: PointOffsetType) -> &[VectorElementType] {
        self.vectors.get(key)
    }
}

impl VectorStorage for SimpleDenseVectorStorage {
    fn vector_dim(&self) -> usize {
        self.dim
    }

    fn distance(&self) -> Distance {
        self.distance
    }

    fn is_on_disk(&self) -> bool {
        false
    }

    fn total_vector_count(&self) -> usize {
        self.vectors.len()
    }

    fn get_vector(&self, key: PointOffsetType) -> CowVector {
        self.get_dense(key).into()
    }

    fn insert_vector(&mut self, key: PointOffsetType, vector: VectorRef) -> OperationResult<()> {
        let vector = vector.try_into()?;
        self.vectors.insert(key, vector)?;
        self.set_deleted(key, false);
        self.update_stored(key, false, Some(vector))?;
        Ok(())
    }

    fn update_from(
        &mut self,
        other: &VectorStorageEnum,
        other_ids: &mut dyn Iterator<Item = PointOffsetType>,
        stopped: &AtomicBool,
    ) -> OperationResult<Range<PointOffsetType>> {
        let start_index = self.vectors.len() as PointOffsetType;
        for point_id in other_ids {
            check_process_stopped(stopped)?;
            // Do not perform preprocessing - vectors should be already processed
            let other_vector = other.get_vector(point_id);
            let other_vector = other_vector.as_vec_ref().try_into()?;
            let other_deleted = other.is_deleted_vector(point_id);
            let new_id = self.vectors.push(other_vector)?;
            self.set_deleted(new_id, other_deleted);
            self.update_stored(new_id, other_deleted, Some(other_vector))?;
        }
        let end_index = self.vectors.len() as PointOffsetType;
        Ok(start_index..end_index)
    }

    fn flusher(&self) -> Flusher {
        self.db_wrapper.flusher()
    }

    fn files(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn delete_vector(&mut self, key: PointOffsetType) -> OperationResult<bool> {
        let is_deleted = !self.set_deleted(key, true);
        if is_deleted {
            self.update_stored(key, true, None)?;
        }
        Ok(is_deleted)
    }

    fn is_deleted_vector(&self, key: PointOffsetType) -> bool {
        self.deleted.get(key as usize).map(|b| *b).unwrap_or(false)
    }

    fn deleted_vector_count(&self) -> usize {
        self.deleted_count
    }

    fn deleted_vector_bitslice(&self) -> &BitSlice {
        self.deleted.as_bitslice()
    }
}
