use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use hnsw_rs::dist::{Distance as Dist, DistCosine, DistDot, DistHamming, DistHellinger, DistJaccard, DistJeffreys, DistJensenShannon, DistL1, DistL2};

use crate::engine::types::types::{ScoreType, VectorElementType};

#[derive(
Debug, Deserialize, Serialize, JsonSchema, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum Distance {
  Manhatten,
  Euclidean,
  DotProduct,
  Cosine,
  Hamming,
  Jaccard,
  Hellinger,
  Jeffreys,
  JensenShannon,
}

impl Distance {
  pub fn evaluate(&self, a: &[VectorElementType], b: &[VectorElementType]) -> ScoreType {
    match self {
      Distance::Manhatten => DistL1.eval(a, b),
      Distance::Euclidean => DistL2.eval(a, b),
      Distance::DotProduct => DistDot.eval(a, b),
      Distance::Cosine => DistCosine.eval(a, b),
      // Distance::Jaccard => DistJaccard.eval(a, b),
      Distance::Hamming => DistHamming.eval(a, b),
      Distance::Hellinger => DistHellinger.eval(a, b),
      Distance::Jeffreys => DistJeffreys.eval(a, b),
      Distance::JensenShannon => DistJensenShannon.eval(a, b),
      _ => panic!("Distance metric not implemented"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_distance_evaluation() {
    let v1: Vec<VectorElementType> = vec![1.0, 2.0, 3.0];
    let v2: Vec<VectorElementType> = vec![4.0, 5.0, 6.0];

    // Test Manhattan distance
    let manhattan_distance = Distance::Manhatten.evaluate(&v1, &v2);
    let expected_manhattan = 9.0; // |1-4| + |2-5| + |3-6|
    assert_eq!(manhattan_distance, expected_manhattan);

    // Test Euclidean distance
    let euclidean_distance = Distance::Euclidean.evaluate(&v1, &v2);
    let expected_euclidean = ((3.0_f64).sqrt() * 3.0) as ScoreType; // sqrt((1-4)^2 + (2-5)^2 + (3-6)^2)
    assert!((euclidean_distance - expected_euclidean).abs() < 1e-5);

    // Test Dot Product distance
    let dot_product_distance = Distance::DotProduct.evaluate(&v1, &v2);
    let expected_dot_product = 32.0; // 1*4 + 2*5 + 3*6
    assert_eq!(dot_product_distance, expected_dot_product);
  }
}
