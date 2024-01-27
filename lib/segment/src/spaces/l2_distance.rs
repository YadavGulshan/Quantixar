use log::debug;
#[cfg(feature = "stdsimd")]
use packed_simd::*;
#[cfg(feature = "simdeez_f")]
use simdeez::*;
#[cfg(feature = "simdeez_f")]
use simdeez::avx2::*;
#[cfg(feature = "simdeez_f")]
use simdeez::sse2::*;

use crate::spaces::distance::DistanceMetric;
use crate::spaces::metrics::{EuclidMetric, Metric};
use crate::types::distance::{Distance, ScoreType};
use crate::types::vector::VectorElementType;

macro_rules! implement_euclid_metric {
    ($ty:ty) => (
        impl Metric<$ty> for EuclidMetric {
            fn distance() -> Distance {
                DistanceMetric::Euclid
            }

            fn similarity(v1: &[$ty], v2: &[$ty]) -> ScoreType {
                let res: ScoreType = v1.iter().zip(v2.iter()).map(
                    |t| ((*t.0 as f32 - *t.1 as f32)* (*t.0 as f32 - *t.1 as f32))
                ).sum();
                res.sqrt()
            }

            fn preprocess(vector: Vec<$ty>) -> Vec<$ty> {
                vector
            }

            fn postprocess(score: ScoreType) -> ScoreType {
                score
            }
        }
    );
}

implement_euclid_metric!(i32);
implement_euclid_metric!(f64);
implement_euclid_metric!(i64);
implement_euclid_metric!(u32);
implement_euclid_metric!(u16);
implement_euclid_metric!(u8);


fn residual_euclid_distanc(
    v1: &[VectorElementType],
    v2: &[VectorElementType],
) -> VectorElementType {
    let res: VectorElementType = v1.iter().zip(v2.iter()).map(|(a, b)| (a - b) * (a - b)).sum();
    res.sqrt()
}

#[cfg(feature = "stdsimd")]
pub fn euclid_metric_f32_simd(v1: &[f32], v2: &[f32]) -> ScoreType {
    debug!("[STD_SIMD]: Executing euclid_metric_f32_simd");
    let nb_lanes = 16;
    let nb_simd = v1.len() / nb_lanes;
    let simd_lenght = nb_simd * nb_lanes;

    let dist_simd: f32x16 = v1
        .chunks_exact(nb_lanes)
        .map(f32x16::from_slice_aligned)
        .zip(v2.chunks_exact(nb_lanes).map(f32x16::from_slice_aligned))
        .map(|(a, b)| (a - b) * (a - b))
        .sum();

    let mut dist = dist_simd.sum().sqrt();
    let dist_residual = residual_euclid_distanc(&v1[simd_lenght..], &v2[simd_lenght..]);
    dist += dist_residual;
    dist
}


#[cfg(feature = "simdeez_f")]
unsafe fn euclid_distance_f32_simdeez_f<S: Simd>(v1: &[f32], v2: &[f32]) -> f32 {
    debug!("[SIMDEEZ]: Executing euclid_distance_f32_simdeez_f");
    let nb_simd = v1.len() / S::VF32_WIDTH;
    let simd_length = nb_simd * S::VF32_WIDTH;
    let mut dist_simd = S::setzero_ps();
    let mut i = 0;
    while i < simd_length {
        let a = S::loadu_ps(&v1[i]);
        let b = S::loadu_ps(&v2[i]);
        let mut delta = a - b;
        delta *= delta;
        dist_simd = dist_simd + delta;
        //
        i += S::VF32_WIDTH;
    }
    let mut dist = S::horizontal_add_ps(dist_simd);
    for i in simd_length..v1.len() {
        let delta = v1[i] - v2[i];
        dist += delta * delta;
    }
    dist.sqrt()
}

pub fn euclid_distance_f32(v1: &[f32], v2: &[f32]) -> ScoreType {
    debug!("[SIMPLE]: Executing euclid_distance_f32");
    let result: ScoreType = v1.iter().zip(v2.iter()).map(|(a, b)| (a - b) * (a - b)).sum();
    result.sqrt()
}

#[cfg(feature = "simdeez_f")]
fn distance_l1_f32_simdeez_f(va: &[f32], vb: &[f32]) -> f32 {
    debug!("[SIMDEEZ]: Executing distance_l1_f32_simdeez_f");
    #[cfg(target_arch = "x86_64")] unsafe {
        if is_x86_feature_detected!("avx")
            && is_x86_feature_detected!("fma")
            && va.len() >= MIN_DIM_SIZE_AVX
        {
            euclid_distance_f32_simdeez_f::<Avx2>(va, vb)
        } else if is_x86_feature_detected!("sse")
            && va.len() >= MIN_DIM_SIZE_SIMD {
            euclid_distance_f32_simdeez_f::<Sse2>(va, vb)
        }
    }
    euclid_distance_f32(va, vb)
}


#[cfg(feature = "stdsimd")]
fn euclid_distance_std_f32(v1: &[f32], v2: &[f32]) -> ScoreType {
    debug!("[STD_SIMD]: Executing euclid_distance_std_f32");
    #[cfg(target_arch = "aarch64")] unsafe {
        if std::arch::aarch64::is_aarch64_feature_detected!("neon")
            && v1.len() >= MIN_DIM_SIZE_SIMD
        {
            euclid_metric_f32_simd(v1, v2)
        }
    }
    euclid_distance_f32(v1, v2)
}

impl Metric<VectorElementType> for EuclidMetric {
    fn distance() -> Distance {
        DistanceMetric::Euclid
    }

    fn similarity(v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        debug!(target: "euclid_metric", "Executing similarity");
        #[cfg(feature = "simdeez_f")] {
            return unsafe { distance_l1_f32_simdeez_f(v1, v2) };
        }
        #[cfg(feature = "stdsimd")] {
            return euclid_distance_std_f32(v1, v2);
        }
    }

    fn preprocess(vector: Vec<VectorElementType>) -> Vec<VectorElementType> {
        vector
    }

    fn postprocess(score: ScoreType) -> ScoreType {
        score
    }
}

mod tests {
    use log::info;
    use crate::spaces::l2_distance::euclid_distance_f32;
    use crate::spaces::metrics::{EuclidMetric, Metric};
    use crate::types::vector::VectorElementType;

    #[test]
    fn test_euclid_metric() {
        info!("Executing test_euclid_metric");
        let v1: Vec<VectorElementType> = vec![1.0, 2.0, 3.0, 4.0];
        let v2: Vec<VectorElementType> = vec![5.0, 6.0, 7.0, 8.0];
        let v3: Vec<VectorElementType> = vec![9.0, 10.0, 11.0, 12.0];

        assert_eq!(EuclidMetric::similarity(&v1, &v2), euclid_distance_f32(&v1, &v2));
        assert_eq!(EuclidMetric::similarity(&v1, &v3), euclid_distance_f32(&v1, &v3));
        assert_eq!(EuclidMetric::similarity(&v2, &v3), euclid_distance_f32(&v2, &v3));
    }
}