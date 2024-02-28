use schemars::JsonSchema;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::engine::types::types::Payload;

#[derive(Deserialize, Debug, Validate, JsonSchema, ToSchema)]
pub struct AddVector {
    pub vectors: Vec<f32>,
    pub payload: Payload,
}

#[derive(Deserialize, Debug, Validate, JsonSchema, ToSchema)]
pub struct SearchVector {
    pub vector: Vec<f32>,
    pub k: usize,
}
