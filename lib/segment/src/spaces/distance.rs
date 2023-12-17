use num_derive::FromPrimitive;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Deserialize, Serialize, JsonSchema, Clone, Copy, FromPrimitive, PartialEq, Eq, Hash,
)]
pub enum DistanceMetric {
    CityBlock,
    Euclid,
    Dot,
    Cosine,
    Hamming,
    Jaccard,
    Hellinger,
    Jeffreys,
    JensenShannon
}
