use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Debug, Validate, JsonSchema)]
pub struct AddVector {
    pub vectors: Vec<f32>,
    pub payload: String,
}

#[derive(Deserialize, Debug)]
pub struct SearchVector {
    pub vector: Vec<f64>,
    pub k: usize,
}
