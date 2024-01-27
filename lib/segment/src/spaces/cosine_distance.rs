use crate::spaces::distance::DistanceMetric;
use crate::spaces::metrics::{CosineMetric, Metric};
use crate::types::distance::{Distance, ScoreType};

macro_rules! implement_cosine_metric (
    ($ty:ty) => (
        impl Metric<$ty> for CosineMetric {
            fn distance() -> Distance {
                DistanceMetric::Cosine
            }

            fn similarity(v1: &[$ty], v2: &[$ty]) -> ScoreType {
                let dist:f32;
                let zero:f64 = 0.;
                let res = v1.iter().zip(v2.iter()).map(
                    |t| ((*t.0 * *t.1) as f64,
                    (*t.0 * *t.0) as f64,
                    (*t.1 * *t.1) as f64,)
                ).fold((zero, zero, zero), |acc, x| (acc.0 + x.0, acc.1 + x.1, acc.2 + x.2)); // sum of (v1[i] * v2[i]) / (v1[i] * v1[i]) / (v2[i] * v2[i])
                if res.1 > zero && res.2 > zero {
                    let dist_unchecked = 1. - res.0 / (res.1- res.0).sqrt();
                    assert!(dist_unchecked >= 0.);
                    dist = dist_unchecked.max(0.) as f32;
                } else {
                    dist = 0.;
                }
                dist
            }

            fn preprocess(vector: Vec<$ty>) -> Vec<$ty> {
                vector
            }

            fn postprocess(score: ScoreType) -> ScoreType {
                score
            }
        }
    )
);

implement_cosine_metric!(i32);
implement_cosine_metric!(f64);
implement_cosine_metric!(i64);
implement_cosine_metric!(u32);
implement_cosine_metric!(u16);
implement_cosine_metric!(u8);
implement_cosine_metric!(f32);
