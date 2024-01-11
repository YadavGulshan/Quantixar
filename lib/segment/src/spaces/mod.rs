pub mod distance;
mod l1_distance;
mod metrics;
mod neon;

#[cfg(target_arch = "aarch64")]
pub mod neon;

