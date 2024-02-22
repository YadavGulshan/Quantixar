use std::mem::size_of;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use atomic_refcell::AtomicRefCell;
use bitvec::prelude::{BitSlice, BitVec};
use log::debug;
use parking_lot::RwLock;
use rocksdb::DB;
use serde::{Deserialize, Serialize};

use hnsw_rs::dist::DistKind;

use crate::common::operation_error::{OperationError, OperationResult};
use crate::engine::storage::rocksdb::Flusher;
use crate::engine::storage::rocksdb::rocksdb_wrapper::DatabaseColumnWrapper;
use crate::engine::storage::vector::base::{DenseVectorStorage, VectorStorage, VectorStorageEnum};
use crate::engine::storage::vector::bitvec::bitvec_set_deleted;
use crate::engine::storage::vector::chunked_vectors::ChunkedVectors;
use crate::engine::types::cow_vector::CowVector;
use crate::engine::types::types::{PointOffsetType, VectorElementType};
use crate::engine::types::vector::VectorRef;

/// In-memory vector storage with on-update persistence using `store`
pub struct SimpleDenseVectorStorage {
  dim: usize,
  distance: DistKind,
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
  distance: DistKind,
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
    todo!()
  }

  fn distance(&self) -> DistKind {
    todo!()
  }

  fn is_on_disk(&self) -> bool {
    todo!()
  }

  fn total_vector_count(&self) -> usize {
    todo!()
  }

  fn get_vector(&self, key: PointOffsetType) -> CowVector {
    todo!()
  }

  fn insert_vector(&mut self, key: PointOffsetType, vector: VectorRef) -> OperationResult<()> {
    todo!()
  }

  fn update_from(&mut self, other: &VectorStorageEnum, other_ids: &mut dyn Iterator<Item=PointOffsetType>, stopped: &AtomicBool) -> OperationResult<Range<PointOffsetType>> {
    todo!()
  }

  fn flusher(&self) -> Flusher {
    todo!()
  }

  fn files(&self) -> Vec<PathBuf> {
    todo!()
  }

  fn delete_vector(&mut self, key: PointOffsetType) -> OperationResult<bool> {
    todo!()
  }

  fn is_deleted_vector(&self, key: PointOffsetType) -> bool {
    todo!()
  }

  fn deleted_vector_count(&self) -> usize {
    todo!()
  }

  fn deleted_vector_bitslice(&self) -> &BitSlice {
    todo!()
  }
}
