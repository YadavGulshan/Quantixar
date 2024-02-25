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

impl VectorIndexEnum {
  pub fn is_index(&self) -> bool {
    match self {
      _ => true,
    }
  }
}

impl VectorIndex for VectorIndexEnum {
  fn search(
    &self,
    vectors: &[&QueryVector],
    top: usize,
    is_stopped: &AtomicBool,
  ) -> OperationResult<Vec<Neighbour>> {
    match self {
      VectorIndexEnum::HNSWMmap(v) => {
        println!("vector {:?}", vectors.into());
        let neighbours = v.search(vectors.into(), top, 24);
        Ok(neighbours)
      }
    }
  }

  fn build_index(&mut self, stopped: &AtomicBool) -> OperationResult<()> {
    match self {
      VectorIndexEnum::HNSWMmap(v) => v.build_index(stopped),
    }
  }

  fn files(&self) -> Vec<PathBuf> {
    match self {
      VectorIndexEnum::HNSWMmap(v) => v.files(),
    }
  }

  fn indexed_vector_count(&self) -> usize {
    match self {
      VectorIndexEnum::HNSWMmap(v) => v.indexed_vector_count(),
    }
  }

  fn update_vector(&mut self, id: PointOffsetType, vector: VectorRef) -> OperationResult<()> {
    match self {
      VectorIndexEnum::HNSWMmap(v) => v.update_vector(id, vector),
    }
  }
}
