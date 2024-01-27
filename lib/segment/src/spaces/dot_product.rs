#[cfg(feature = "stdsimd")]
use packed_simd::*;
#[cfg(feature = "simdeez_f")]
use simdeez::*;
#[cfg(feature = "simdeez_f")]
use simdeez::avx2::*;
#[cfg(feature = "simdeez_f")]
use simdeez::sse2::*;

use crate::spaces::metrics::{DotProductMetric, Metric};
use crate::types::distance::{Distance, ScoreType};
use crate::types::vector::VectorElementType;

macro_rules! implement_dot_distance (
    ($ty:ty) => (
        impl Metric<$ty> for DotProductMetric {
            fn distance() -> Distance {
                DistanceMetric::Dot
            }

            fn similarity(v1: &[$ty], v2: &[$ty]) -> ScoreType {
                let zero:f32 = 0f32;
                let dot = va.iter().zip(vb.iter()).map(|t| (*t.0 * *t.1) as f32).fold(0., |acc , t| (acc + t));
                assert(dot <= 1.);
                return  1. - dot;
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


#[cfg(feature = "simdeez_f")]
unsafe fn distance_dot_f32<S: Simd>(v1: &[f32], v2: &[f32]) -> f32 {
    let mut i = 0;
    let mut dot_simd = S::setzero_ps();
    let nb_simd = v1.len() / S::VF32_WIDTH;
    let simd_length = nb_simd * S::VF32_WIDTH;
    while i < simd_length {
        let a = S::loadu_ps(&v1[i]);
        let b = S::loadu_ps(&v2[i]);
        let delta = a * b;
        dot_simd += delta;
        i += S::VF32_WIDTH;
    }
    let mut dot = S::horizontal_add_ps(dot_simd);
    for i in simd_length..v1.len() {
        dot += v1[i] * v2[i];
    }
    assert!(dot <= 1.000002);
    (1. - dot).max(0.)
}

#[cfg(feature = "simdeez_f")]
#[target_feature(enable = "avx2")]
unsafe fn distance_dot_f32_avx2(va: &[f32], vb: &[f32]) -> f32 {
    distance_dot_f32::<Avx2>(va, vb)
}


#[cfg(feature = "simdeez_f")]
#[target_feature(enable = "sse2")]
unsafe fn distance_dot_f32_sse2(va: &[f32], vb: &[f32]) -> f32 {
    distance_dot_f32::<Sse2>(va, vb)
}

impl Metric<VectorElementType> for DotProductMetric {
    fn distance() -> Distance {
        Distance::Dot
    }

    fn similarity(v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        #[cfg(feature = "simdeez_f")] {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
                if is_x86_feature_detected!("avx2") {
                    return unsafe { distance_dot_f32_avx2(v1, v2) };
                } else if is_x86_feature_detected!("sse2") {
                    return unsafe { distance_dot_f32_sse2(v1, v2) };
                }
            }
        }
        let dot = 1. - v1.iter().zip(v2.iter()).map(|t| (*t.0 * *t.1) as f32).fold(0., |acc, t| (acc + t));
        assert!(dot >= 0.);
        dot
    }

    fn preprocess(vector: Vec<VectorElementType>) -> Vec<VectorElementType> {
        vector
    }

    fn postprocess(score: ScoreType) -> ScoreType {
        score
    }
}