#[cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
))]
use crate::spaces::MIN_DIM_SIZE_SIMD;
use crate::types::distance::{Distance, ScoreType};
use crate::types::vector::{VectorElementType};

use super::distance::{dot_similarity, euclid_similarity};

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use super::neon::dot_neon_similarity;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use crate::spaces::neon::euclidian_neon_similarity;

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

impl Metric<VectorElementType> for EuclidMetric {
    fn distance() -> Distance {
        Distance::Euclid
    }

    fn similarity(v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            if std::arch::is_aarch64_feature_detected!("neon") && v1.len() >= MIN_DIM_SIZE_SIMD {
                return unsafe { euclidian_neon_similarity(v1, v2) };
            }
        }
        euclid_similarity(v1, v2)
    }

    fn preprocess(vector: Vec<VectorElementType>) -> Vec<VectorElementType> {
        vector
    }

    fn postprocess(score: ScoreType) -> ScoreType {
        score
    }
}

impl Metric<VectorElementType> for DotProductMetric {
    fn distance() -> Distance {
        Distance::Dot
    }

    fn similarity(v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            if std::arch::is_aarch64_feature_detected!("neon") && v1.len() >= MIN_DIM_SIZE_SIMD {
                return unsafe { dot_neon_similarity(v1, v2) };
            }
        }
        dot_similarity(v1, v2)
    }

    fn preprocess(vector: Vec<VectorElementType>) -> Vec<VectorElementType> {
        vector
    }

    fn postprocess(score: ScoreType) -> ScoreType {
        score
    }
}
