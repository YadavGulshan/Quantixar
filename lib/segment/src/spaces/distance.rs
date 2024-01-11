use common::types::ScoreType;
use num_derive::FromPrimitive;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::vector::VectorElementType;

#[derive(
    Debug, Deserialize, Serialize, JsonSchema, Clone, Copy, FromPrimitive, PartialEq, Eq, Hash,
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
        /// zip() returns an iterator over pairs of elements from two iterators.
        .zip(v2.iter().copied())
        .map(|(x, y)| (x - y).powi(2))
        .sum();
    -result
}
