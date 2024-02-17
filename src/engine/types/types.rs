use std::cmp::Ordering;

use ordered_float::OrderedFloat;

/// Type of vector matching score
pub type ScoreType = f32;
/// Type of point index inside a segment
pub type PointOffsetType = u32;

pub type VectorElementType = f32;


pub const DEFAULT_VECTOR_NAME: &str = "";

pub type DenseVector = Vec<VectorElementType>;


#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct ScoredPointOffset {
  pub idx: PointOffsetType,
  pub score: ScoreType,
}

impl Eq for ScoredPointOffset {}

impl Ord for ScoredPointOffset {
  fn cmp(&self, other: &Self) -> Ordering {
    OrderedFloat(self.score).cmp(&OrderedFloat(other.score))
  }
}

impl PartialOrd for ScoredPointOffset {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
