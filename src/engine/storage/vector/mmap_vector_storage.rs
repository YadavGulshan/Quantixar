use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::io::Write;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use atomic_refcell::AtomicRefCell;
use bitvec::prelude::BitSlice;
use cgroups_rs::Controller;

use hnsw_rs::dist::DistKind;
use memory::mmap_ops;

use crate::common::operation_error::{check_process_stopped, OperationResult};
use crate::engine::storage::rocksdb::Flusher;
use crate::engine::storage::vector::async_common::get_async_scorer;
use crate::engine::storage::vector::base::{DenseVectorStorage, VectorStorage, VectorStorageEnum};
use crate::engine::storage::vector::mmap_vector::MmapVectors;
use crate::engine::types::cow_vector::CowVector;
use crate::engine::types::types::{DenseVector, PointOffsetType, VectorElementType};
use crate::engine::types::vector::VectorRef;

const VECTORS_PATH: &str = "matrix.dat";
const DELETED_PATH: &str = "deleted.dat";

pub struct MemmapVectorStorage {
  vectors_path: PathBuf,
  deleted_path: PathBuf,
  mmap_store: Option<MmapVectors>,
  distance: DistKind,
}


pub fn open_memmap_vector_storage(
  path: &Path,
  dim: usize,
  distance: DistKind,
) -> OperationResult<Arc<AtomicRefCell<VectorStorageEnum>>> {
  open_memmap_vector_storage_with_async_io(path, dim, distance, get_async_scorer())
}

pub fn open_memmap_vector_storage_with_async_io(
  path: &Path,
  dim: usize,
  distance: DistKind,
  with_async_io: bool,
) -> OperationResult<Arc<AtomicRefCell<VectorStorageEnum>>> {
  create_dir_all(path)?;

  let vectors_path = path.join(VECTORS_PATH);
  let deleted_path = path.join(DELETED_PATH);
  let mmap_store = MmapVectors::open(&vectors_path, &deleted_path, dim, with_async_io)?;

  Ok(Arc::new(AtomicRefCell::new(VectorStorageEnum::Memmap(
    Box::new(MemmapVectorStorage {
      vectors_path,
      deleted_path,
      mmap_store: Some(mmap_store),
      distance,
    }),
  ))))
}


impl MemmapVectorStorage {
  pub fn prefault_mmap_pages(&self) -> Option<mmap_ops::PrefaultMmapPages> {
    Some(
      self.mmap_store
              .as_ref()?
              .prefault_mmap_pages(&self.vectors_path),
    )
  }

  pub fn get_mmap_vectors(&self) -> &MmapVectors {
    self.mmap_store.as_ref().unwrap()
  }

  pub fn has_async_reader(&self) -> bool {
    self.mmap_store
            .as_ref()
            .map(|x| x.has_async_reader())
            .unwrap_or(false)
  }
}

impl DenseVectorStorage for MemmapVectorStorage {
  fn get_dense(&self, key: PointOffsetType) -> &[VectorElementType] {
    self.mmap_store.as_ref().unwrap().get_vector(key)
  }
}

impl VectorStorage for MemmapVectorStorage {
  fn vector_dim(&self) -> usize {
    self.mmap_store.as_ref().unwrap().dim
  }

  fn distance(&self) -> DistKind {
    self.distance.clone() // Not good for performance, TODO: fix
  }

  fn is_on_disk(&self) -> bool {
    true
  }

  fn total_vector_count(&self) -> usize {
    self.mmap_store.as_ref().unwrap().num_vectors
  }

  fn get_vector(&self, key: PointOffsetType) -> CowVector {
    self.get_dense(key).into()
  }

  fn insert_vector(&mut self, _key: PointOffsetType, _vector: VectorRef) -> OperationResult<()> {
    panic!("Can't directly update vector in mmap storage")
  }

  fn update_from(
    &mut self,
    other: &VectorStorageEnum,
    other_ids: &mut dyn Iterator<Item=PointOffsetType>,
    stopped: &AtomicBool,
  ) -> OperationResult<Range<PointOffsetType>> {
    let dim = self.vector_dim();
    let start_index = self.mmap_store.as_ref().unwrap().num_vectors as PointOffsetType;
    let mut end_index = start_index;

    let with_async_io = self
            .mmap_store
            .take()
            .map(|x| x.has_async_reader())
            .unwrap_or(get_async_scorer());

    // Extend vectors file, write other vectors into it
    let mut vectors_file = open_append(&self.vectors_path)?;
    let mut deleted_ids = vec![];
    for id in other_ids {
      check_process_stopped(stopped)?;
      let vector: DenseVector = other.get_vector(id).try_into()?;
      let raw_bites = mmap_ops::transmute_to_u8_slice(&vector);
      vectors_file.write_all(raw_bites)?;
      end_index += 1;

      // Remember deleted IDs so we can propagate deletions later
      if other.is_deleted_vector(id) {
        deleted_ids.push((start_index + id) as PointOffsetType);
      }
    }
    vectors_file.flush()?;
    drop(vectors_file);

    // Load store with updated files
    self.mmap_store.replace(MmapVectors::open(
      &self.vectors_path,
      &self.deleted_path,
      dim,
      with_async_io,
    )?);

    // Flush deleted flags into store
    // We must do that in the updated store, and cannot do it in the previous loop. That is
    // because the file backing delete storage must be resized, and for that we'd need to know
    // the exact number of vectors beforehand. When opening the store it is done automatically.
    let store = self.mmap_store.as_mut().unwrap();
    for id in deleted_ids {
      check_process_stopped(stopped)?;
      store.delete(id);
    }

    Ok(start_index..end_index)
  }

  fn flusher(&self) -> Flusher {
    match &self.mmap_store {
      Some(mmap_store) => mmap_store.flusher(),
      None => Box::new(|| Ok(())),
    }
  }

  fn files(&self) -> Vec<PathBuf> {
    vec![self.vectors_path.clone(), self.deleted_path.clone()]
  }

  fn delete_vector(&mut self, key: PointOffsetType) -> OperationResult<bool> {
    Ok(self.mmap_store.as_mut().unwrap().delete(key))
  }

  fn is_deleted_vector(&self, key: PointOffsetType) -> bool {
    self.mmap_store.as_ref().unwrap().is_deleted_vector(key)
  }

  fn deleted_vector_count(&self) -> usize {
    self.mmap_store.as_ref().unwrap().deleted_count
  }

  fn deleted_vector_bitslice(&self) -> &BitSlice {
    self.mmap_store.as_ref().unwrap().deleted_vector_bitslice()
  }
}


/// Open a file shortly for appending
fn open_append<P: AsRef<Path>>(path: P) -> io::Result<File> {
  OpenOptions::new()
          .read(false)
          .write(false)
          .append(true)
          .create(false)
          .open(path)
}
