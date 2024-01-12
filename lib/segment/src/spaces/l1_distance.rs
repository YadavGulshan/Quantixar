use cfg_if;
#[cfg(feature = "stdsimd")]
use packed_simd::*;

#[cfg(feature = "simdeez_f")]
use simdeez::*;

#[cfg(feature = "simdeez_f")]
#[cfg(target_arch = "x86_64")]
use simdeez::avx2::*;

#[cfg(target_feature = "neon")]
use std::arch::aarch64::*;

use crate::spaces::distance::DistanceMetric;
use crate::spaces::metrics::{CityBlockMetric, Metric};
use crate::types::distance::{Distance, ScoreType};
use crate::types::vector::VectorElementType;

macro_rules! implement_city_block_metric (
    ($ty:ty) => (
        impl Metric<$ty> for CityBlockMetric {
            fn distance() -> Distance {
                DistanceMetric::CityBlock
            }

            fn similarity(v1: &[$ty], v2: &[$ty]) -> ScoreType {
                v1.iter().zip(v2.iter()).map(
                    |t| (*t.0 as f32 - *t.1 as f32).abs()
                ).sum()
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

implement_city_block_metric!(i32);
implement_city_block_metric!(f64);
implement_city_block_metric!(i64);
implement_city_block_metric!(u32);
implement_city_block_metric!(u16);
implement_city_block_metric!(u8);

#[cfg(feature = "stdsimd")]
fn residual_city_block_distance(
    v1: &[VectorElementType],
    v2: &[VectorElementType],
) -> VectorElementType {
    v1.iter().zip(v2.iter()).map(|(a, b)| (a - b).abs()).sum()
}

#[cfg(feature = "stdsimd")]
pub fn city_block_distance_f32_simd(v1: &[f32], v2: &[f32]) -> ScoreType {
    let nb_lanes = 16;
    let nb_simd = v1.len() / nb_lanes;
    let simd_length = nb_simd * nb_lanes;

    let dist_simd = v1
        .chunks_exact(nb_lanes)
        .map(f32x16::from_slice_aligned)
        .zip(v2.chunks_exact(nb_lanes).map(f32x16::from_slice_unaligned))
        .map(|(a, b)| (a - b).abs())
        .sum();
    let mut dist = dist_simd.sum();
    let dist_residual = residual_city_block_distance(&v1[simd_length..], &v2[simd_length..]);
    dist += dist_residual;

    dist
}

#[cfg(all(feature = "simdeez_f", target_arch = "x86_64"))]
unsafe fn distance_l1_f32<S: Simd>(v1: &[f32], v2: &[f32]) -> f32 {
    assert_eq!(v1.len(), v2.len());
    let mut dist_simd = S::setzero_ps();
    let nb_simd = v1.len() / S::VF32_WIDTH;
    let simd_length = nb_simd * S::VF32_WIDTH;
    let mut i = 0;
    while i < simd_length {
        let a = S::loadu_ps(&v1[i]);
        let b = S::loadu_ps(&v2[i]);
        let delta = S::abs_ps(a - b);
        dist_simd += delta;
        //
        i += S::VF32_WIDTH;
    }
    let mut dist: f32 = S::horizontal_add_ps(dist_simd);
    let dist_residual = residual_city_block_distance(&v1[simd_length..], &v2[simd_length..]);
    dist += dist_residual;

    dist
}

#[cfg(feature = "simdeez_f")]
#[target_feature(enable = "avx2")]
#[cfg(target_arch = "x86_64")]
unsafe fn distance_l1_f32_avx2(va: &[f32], vb: &[f32]) -> f32 {
    distance_l1_f32::<Avx2>(va, vb)
}

impl Metric<VectorElementType> for CityBlockMetric {
    fn distance() -> Distance {
        DistanceMetric::CityBlock
    }

    fn similarity(v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        cfg_if::cfg_if! {
        if #[cfg(feature = "simdeez_f")] {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
                if is_x86_feature_detected!("avx2") {
                    return unsafe {distance_l1_f32_avx2(va,vb)};
                }
                else {
                    assert_eq!(v1.len(), v1.len());
                    v1.iter().zip(v1.iter()).map(|t| (*t.0 as f32- *t.1 as f32).abs()).sum()
                }
            }
           #[cfg(target_arch = "aarch64")] {
                else unsafe{
                    let n = v1.len();
                    let m = n - (n % 16);
                    let mut ptr1: *const f32 = v1.as_ptr();
                    let mut ptr2: *const f32 = v2.as_ptr();
                    //     this will duplicate float value to all lanes of 128-bit register of NEON
                    let mut sum1 = vdupq_n_f32(0.);
                    let mut sum2 = vdupq_n_f32(0.);
                    let mut sum3 = vdupq_n_f32(0.);
                    let mut sum4 = vdupq_n_f32(0.);

                    let mut i: usize = 0;
                    while i < m {
                        let sub1 = vsubq_f32(vld1q_f32(ptr1), vld1q_f32(ptr2));
                        sum1 = vaddq_f32(sum1, vabsq_f32(sub1));

                        let sub2 = vsubq_f32(vld1q_f32(ptr1.add(4)), vld1q_f32(ptr2.add(4)));
                        sum2 = vaddq_f32(sum2, vabsq_f32(sub2));

                        let sub3 = vsubq_f32(vld1q_f32(ptr1.add(8)), vld1q_f32(ptr2.add(8)));
                        sum3 = vaddq_f32(sum3, vabsq_f32(sub3));

                        let sub4 = vsubq_f32(vld1q_f32(ptr1.add(12)), vld1q_f32(ptr2.add(12)));
                        sum4 = vaddq_f32(sum4, vabsq_f32(sub4));

                        ptr1 = ptr1.add(16);
                        ptr2 = ptr2.add(16);
                        i += 16;
                    }
                    let mut result = vaddvq_f32(sum1) + vaddvq_f32(sum2) + vaddvq_f32(sum3) + vaddvq_f32(sum4);
                    for i in 0..n - m {
                        result += (*ptr1.add(i) - *ptr2.add(i)).abs();
                    }
                    -result
                }
            }
        }
        else if #[cfg(feature = "stdsimd")] {
            distance_l1_f32_simd(v1,v2)
            }
        else {
            v1.iter().zip(v2.iter()).map(|t| (*t.0 as f32- *t.1 as f32).abs()).sum()
            }
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
    use crate::{
        spaces::metrics::{CityBlockMetric, Metric},
        types::vector::VectorElementType,
    };

    #[test]
    fn test_city_block_metric() {
        let v1: Vec<VectorElementType> = vec![1.0, 2.0, 3.0, 4.0];
        let v2: Vec<VectorElementType> = vec![5.0, 6.0, 7.0, 8.0];
        let v3: Vec<VectorElementType> = vec![9.0, 10.0, 11.0, 12.0];

        assert_eq!(CityBlockMetric::similarity(&v1, &v2), 16.0);
        assert_eq!(CityBlockMetric::similarity(&v1, &v3), 32.0);
        assert_eq!(CityBlockMetric::similarity(&v2, &v3), 16.0);
    }
}
