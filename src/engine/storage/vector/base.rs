use crate::engine::types::cow_vector::CowVector;
use crate::engine::types::types::PointOffsetType;

pub trait VectorStorage {
  fn vector_dim(&self) -> usize;
  fn is_on_disk(&self) -> bool;
  fn total_vector_count(&self) -> usize;
  fn get_vector(&self, key: PointOffsetType) -> CowVector;
}
