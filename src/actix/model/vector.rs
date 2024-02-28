use schemars::JsonSchema;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Debug, Validate, JsonSchema, ToSchema)]
pub struct AddVector {
    pub vectors: Vec<f32>,
    pub payload: String,
}

#[derive(Deserialize, Debug, Validate, JsonSchema, ToSchema)]
pub struct SearchVector {
    pub vector: Vec<f32>,
    pub k: usize,
}
