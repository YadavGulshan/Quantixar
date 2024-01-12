#[cfg(target_feature = "neon")]
use std::arch::aarch64::*;

use common::types::ScoreType;

use crate::types::vector::VectorElementType;

#[cfg(target_feature = "neon")]
pub unsafe fn euclidian_neon_similarity(
    v1: &[VectorElementType],
    v2: &[VectorElementType],
) -> ScoreType {
    let n = v1.len();
    let m = n - (n % 16);
    let mut ptr1: *const f32 = v1.as_ptr();
    let mut ptr2: *const f32 = v2.as_ptr();
    let mut sum1 = vdupq_n_f32(0.);
    let mut sum2 = vdupq_n_f32(0.);
    let mut sum3 = vdupq_n_f32(0.);
    let mut sum4 = vdupq_n_f32(0.);

    let mut i: usize = 0;
    while i < m {
        let sub1 = vsubq_f32(vld1q_f32(ptr1), vld1q_f32(ptr2));
        sum1 = vfmaq_f32(sum1, sub1, sub1);

        let sub2 = vsubq_f32(vld1q_f32(ptr1.add(4)), vld1q_f32(ptr2.add(4)));
        sum2 = vfmaq_f32(sum2, sub2, sub2);

        let sub3 = vsubq_f32(vld1q_f32(ptr1.add(8)), vld1q_f32(ptr2.add(8)));
        sum3 = vfmaq_f32(sum3, sub3, sub3);

        let sub4 = vsubq_f32(vld1q_f32(ptr1.add(12)), vld1q_f32(ptr2.add(12)));
        sum4 = vfmaq_f32(sum4, sub4, sub4);

        ptr1 = ptr1.add(16);
        ptr2 = ptr2.add(16);
        i += 16;
    }
    let mut result = vaddvq_f32(sum1) + vaddvq_f32(sum2) + vaddvq_f32(sum3) + vaddvq_f32(sum4);
    for i in 0..n - m {
        result += (*ptr1.add(i) - *ptr2.add(i)).powi(2);
    }
    -result
}

#[cfg(target_feature = "neon")]
pub unsafe fn dot_neon_similarity(v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
    let n = v1.len();
    let m = n - (n % 16);
    let mut ptr1: *const f32 = v1.as_ptr();
    let mut ptr2: *const f32 = v2.as_ptr();
    let mut sum1 = vdupq_n_f32(0.);
    let mut sum2 = vdupq_n_f32(0.);
    let mut sum3 = vdupq_n_f32(0.);
    let mut sum4 = vdupq_n_f32(0.);

    let mut i: usize = 0;
    while i < m {
        sum1 = vfmaq_f32(sum1, vld1q_f32(ptr1), vld1q_f32(ptr2));
        sum2 = vfmaq_f32(sum2, vld1q_f32(ptr1.add(4)), vld1q_f32(ptr2.add(4)));
        sum3 = vfmaq_f32(sum3, vld1q_f32(ptr1.add(8)), vld1q_f32(ptr2.add(8)));
        sum4 = vfmaq_f32(sum4, vld1q_f32(ptr1.add(12)), vld1q_f32(ptr2.add(12)));
        ptr1 = ptr1.add(16);
        ptr2 = ptr2.add(16);
        i += 16;
    }
    let mut result = vaddvq_f32(sum1) + vaddvq_f32(sum2) + vaddvq_f32(sum3) + vaddvq_f32(sum4);
    for i in 0..n - m {
        result += (*ptr1.add(i)) * (*ptr2.add(i));
    }
    result
}

#[cfg(test)]
mod test {
    #[cfg(target_feature = "neon")]
    #[test]
    fn test_neon_spaces() {
        use super::*;
        use crate::spaces::distance::euclid_similarity;

        if std::arch::is_aarch64_feature_detected!("neon") {
            let v1: Vec<f32> = vec![
                13., 16., 19., 22., 25., 28., 31., 34., 37., 40., 43., 46., 49., 52., 55., 58.,
            ];
            let v2: Vec<f32> = vec![
                12., 15., 18., 21., 24., 27., 30., 33., 36., 39., 42., 45., 48., 51., 54., 57.,
            ];

            let euclid_simd = unsafe { euclidian_neon_similarity(&v1, &v2) };
            let euclid = euclid_similarity(&v1, &v2);
            assert_eq!(euclid_simd, euclid);
        } else {
            println!("neon not supported");
        }
    }
}
