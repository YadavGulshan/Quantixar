use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use hnsw_rs::dist::DistL2;
use hnsw_rs::hnsw::Neighbour;
use hnsw_rs::prelude::Hnsw;

use crate::common::operation_error::OperationResult;
use crate::engine::types::types::{PointOffsetType, VectorElementType};
use crate::engine::types::vector::{QueryVector, VectorRef};

pub trait VectorIndex {
  fn search(
    &self,
    vectors: &[&QueryVector],
    top: usize,
    is_stopped: &AtomicBool,
  ) -> OperationResult<Vec<Neighbour>>;

  fn build_index(&mut self, stopped: &AtomicBool) -> OperationResult<()>;
  fn files(&self) -> Vec<PathBuf>;
  fn indexed_vector_count(&self) -> usize;
  fn update_vector(&mut self, id: PointOffsetType, vector: VectorRef) -> OperationResult<()>;
}

pub enum VectorIndexEnum<'a> {
  HNSWMmap(Hnsw<'a, VectorElementType, DistL2>),
}

impl<'a> VectorIndexEnum<'a> {
  pub fn is_index(&self) -> bool {
    match self {
      _ => true,
    }
  }
}

impl<'a> VectorIndex for VectorIndexEnum<'a> {
  fn search(
    &self,
    vectors: &[&QueryVector],
    top: usize,
    is_stopped: &AtomicBool,
  ) -> OperationResult<Vec<Neighbour>> {
    match self {
      VectorIndexEnum::HNSWMmap(v) => {
        let f32_vectors: Vec<VectorElementType> = vectors.iter()
                .flat_map(|qv| qv.as_ref()) // Assuming QueryVector implements AsRef<[f32]>
                .copied()
                .collect();

        let neighbours = v.search(f32_vectors.as_slice(), top, 24);
        Ok(neighbours)
      }
    }
  }

  fn build_index(&mut self, stopped: &AtomicBool) -> OperationResult<()> {
    todo!()
  }

  fn files(&self) -> Vec<PathBuf> {
    todo!()
  }

  fn indexed_vector_count(&self) -> usize {
    todo!()
  }

  fn update_vector(&mut self, id: PointOffsetType, vector: VectorRef) -> OperationResult<()> {
    todo!()
  }
}
