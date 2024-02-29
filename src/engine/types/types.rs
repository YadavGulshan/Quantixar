use std::cmp::Ordering;

use ordered_float::OrderedFloat;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::utils::remove_value_from_json_map;

/// Type of vector matching score
pub type ScoreType = f32;
/// Type of point index inside a segment
pub type PointOffsetType = u32;

pub type VectorElementType = f32;

pub const DEFAULT_VECTOR_NAME: &str = "";

pub type DenseVector = Vec<VectorElementType>;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Payload(pub Map<String, Value>);

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct ScoredPointOffset {
    pub idx: PointOffsetType,
    pub score: ScoreType,
}

impl Eq for ScoredPointOffset {}

impl Ord for ScoredPointOffset {
    fn cmp(&self, other: &Self) -> Ordering {
        OrderedFloat(self.score).cmp(&OrderedFloat(other.score))
    }
}

impl PartialOrd for ScoredPointOffset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Payload {
    pub fn merge(&mut self, value: &Payload) {
        for (key, value) in &value.0 {
            match value {
                Value::Null => self.0.remove(key),
                _ => self.0.insert(key.to_owned(), value.to_owned()),
            };
        }
    }

    pub fn remove(&mut self, path: &str) -> Vec<Value> {
        remove_value_from_json_map(path, &mut self.0)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn iter(&self) -> serde_json::map::Iter {
        self.0.iter()
    }
}

impl Default for Payload {
    fn default() -> Self {
        Payload(Map::new())
    }
}
