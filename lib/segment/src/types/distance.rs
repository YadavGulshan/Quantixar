use crate::spaces::distance::DistanceMetric;
use crate::types::vector::{VectorElementType, VectorType};

pub type ScoreType = f32;
pub type Distance = DistanceMetric;

impl Distance {
    pub fn preprocess_vector(&self, vector: VectorType) -> VectorType {
        todo!()
        // match self {
        //     Distance::Cosine => CosineMetric::preprocess(vector),
        //     Distance::Euclid => EuclidMetric::preprocess(vector),
        //     Distance::Dot => DotProductMetric::preprocess(vector),
        //     Distance::Manhattan => ManhattanMetric::preprocess(vector),
        // }
    }

    pub fn postprocess_score(&self, score: ScoreType) -> ScoreType {
        todo!()
        // match self {
        //     Distance::Cosine => CosineMetric::postprocess(score),
        //     Distance::Euclid => EuclidMetric::postprocess(score),
        //     Distance::Dot => DotProductMetric::postprocess(score),
        //     Distance::Manhattan => ManhattanMetric::postprocess(score),
        // }
    }

    pub fn distance_order(&self) -> Order {
        match self {
            Distance::Cosine | Distance::Dot => Order::LargeBetter,
            Distance::Euclid | Distance::CityBlock |
            Distance::Hamming | Distance::Jaccard |
            Distance::Hellinger | Distance::Jeffreys |
            Distance::JensenShannon => Order::SmallBetter,
        }
    }

    /// Checks if score satisfies threshold condition
    pub fn check_threshold(&self, score: ScoreType, threshold: ScoreType) -> bool {
        match self.distance_order() {
            Order::LargeBetter => score > threshold,
            Order::SmallBetter => score < threshold,
        }
    }

    /// Calculates distance between two vectors
    ///
    /// Warn: prefer compile-time generics with `Metric` trait
    pub fn similarity(&self, v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        todo!()
        // match self {
        //     Distance::Cosine => CosineMetric::similarity(v1, v2),
        //     Distance::Euclid => EuclidMetric::similarity(v1, v2),
        //     Distance::Dot => DotProductMetric::similarity(v1, v2),
        //     Distance::Manhattan => ManhattanMetric::similarity(v1, v2),
        // }
    }
}

pub enum Order {
    LargeBetter,
    SmallBetter,
}