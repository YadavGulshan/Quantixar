use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use hnsw_rs::dist::{
    DistCosine, DistDot, DistHamming, DistHellinger, DistJaccard, DistJeffreys, DistJensenShannon,
    DistL1, DistL2, Distance as Dist,
};

use crate::engine::types::types::{ScoreType, VectorElementType};

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, Copy, PartialEq, Eq, Hash)]
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

impl<T: Sync + Send> Dist<T> for Distance
where
    hnsw_rs::dist::DistL1: hnsw_rs::dist::Distance<T>,
    hnsw_rs::dist::DistL2: hnsw_rs::dist::Distance<T>,
    hnsw_rs::dist::DistDot: hnsw_rs::dist::Distance<T>,
    hnsw_rs::dist::DistCosine: hnsw_rs::dist::Distance<T>,
    hnsw_rs::dist::DistHamming: hnsw_rs::dist::Distance<T>,
    hnsw_rs::dist::DistHellinger: hnsw_rs::dist::Distance<T>,
    hnsw_rs::dist::DistJeffreys: hnsw_rs::dist::Distance<T>,
    hnsw_rs::dist::DistJensenShannon: hnsw_rs::dist::Distance<T>,
{
    fn eval(&self, va: &[T], vb: &[T]) -> f32 {
        match self {
            Distance::Manhatten => DistL1.eval(va, vb),
            Distance::Euclidean => DistL2.eval(va, vb),
            Distance::DotProduct => DistDot.eval(va, vb),
            Distance::Cosine => DistCosine.eval(va, vb),
            Distance::Hamming => DistHamming.eval(va, vb),
            Distance::Hellinger => DistHellinger.eval(va, vb),
            Distance::Jeffreys => DistJeffreys.eval(va, vb),
            Distance::JensenShannon => DistJensenShannon.eval(va, vb),
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
        let manhattan_distance = Distance::Manhatten.eval(&v1, &v2);
        let expected_manhattan = 9.0; // |1-4| + |2-5| + |3-6|
        assert_eq!(manhattan_distance, expected_manhattan);

        // Test Euclidean distance
        let euclidean_distance = Distance::Euclidean.eval(&v1, &v2);
        let expected_euclidean = ((3.0_f64).sqrt() * 3.0) as ScoreType; // sqrt((1-4)^2 + (2-5)^2 + (3-6)^2)
        assert!((euclidean_distance - expected_euclidean).abs() < 1e-5);

        // Test Dot Product distance
        let dot_product_distance = Distance::DotProduct.eval(&v1, &v2);
        let expected_dot_product = 32.0; // 1*4 + 2*5 + 3*6
        assert_eq!(dot_product_distance, expected_dot_product);
    }
}
