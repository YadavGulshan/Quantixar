#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use crate::spaces::neon::euclidian_neon_similarity;
use crate::types::distance::{Distance, ScoreType};
use crate::types::vector::VectorElementType;

use super::distance::dot_similarity;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use super::neon::dot_neon_similarity;

pub trait Metric<T: Send + Sync> {
    fn distance() -> Distance;

    /// Greater the value - closer the vectors
    fn similarity(v1: &[T], v2: &[T]) -> ScoreType;

    /// Necessary vector transformations performed before adding it to the collection (like normalization)
    /// If no transformation is needed - returns the same vector
    fn preprocess(vector: Vec<T>) -> Vec<T>;

    /// correct metric score for displaying
    fn postprocess(score: ScoreType) -> ScoreType;
}

#[derive(Clone)]
pub struct CityBlockMetric;

#[derive(Clone)]
pub struct EuclidMetric;

#[derive(Clone)]
pub struct DotProductMetric;

#[derive(Clone)]
pub struct CosineMetric;

#[derive(Clone)]
pub struct HammingMetric;

#[derive(Clone)]
pub struct Jaccard;

#[derive(Clone)]
pub struct Hellinger;

#[derive(Clone)]
pub struct Jeffreys;

#[derive(Clone)]
pub struct JensenShannon;
