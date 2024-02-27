use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

use actix_web::{get, post, web::Data, Responder};
use actix_web_validator::Json;
use hnsw_rs::hnsw::Distance;

use crate::{
    actix::model::vector::AddVector,
    engine::{
        index::hnsw::index::HNSWIndex,
        types::{types::VectorElementType, vector::VectorRef},
    },
};

#[utoipa::path(
    post,
    path = "/vector",
    responses(
        (status = 200, description = "Add Vectors in HSNW",)
    )
)]
#[post("/vector")]
pub async fn add_vector<'a>(
    data: Data<Arc<Mutex<HNSWIndex<'a>>>>,
    operation: Json<AddVector>,
) -> impl Responder {
    let vector: &[VectorElementType] = operation.vectors.as_slice();

    match data.lock().unwrap().add(vector) {
        Ok(()) => "Vector added successfully",
        Err(e) => {
            log::error!("Error adding vector: {}", e);
            "Error adding vector"
        }
    }
}
