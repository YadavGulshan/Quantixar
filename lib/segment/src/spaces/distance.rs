use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::types::distance::ScoreType;

use crate::types::vector::VectorElementType;

#[derive(
   Debug, Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq, Hash,
)]
pub enum DistanceMetric {
   CityBlock,
   Euclid,
   Dot,
   Cosine,
   Hamming,
   Jaccard,
   Hellinger,
   Jeffreys,
   JensenShannon,
}


pub fn euclid_similarity(v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
    let result: ScoreType = v1
        .iter()
        .copied()
        // / zip() returns an iterator over pairs of elements from two iterators.
        .zip(v2.iter().copied())
        .map(|(x, y)| (x - y).powi(2))
        .sum();
    -result
}

pub fn dot_similarity(v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
    v1.iter().zip(v2).map(|(x, y)| x * y).sum()
}
