pub mod distance;
pub mod l1_distance;
pub mod metrics;

#[cfg(target_arch = "aarch64")]
pub mod neon;
pub mod l2_distance;
mod cosine_distance;
mod dot_product;


#[cfg(target_arch = "x86_64")]
const MIN_DIM_SIZE_AVX: usize = 32;

#[cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
))]
/// Minimal size of vector for SIMD processing
const MIN_DIM_SIZE_SIMD: usize = 16;