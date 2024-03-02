use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

use actix_web::{
    get, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use actix_web_validator::Json;
use hnsw_rs::hnsw::Distance;
use serde_json::json;

use crate::{
    actix::model::vector::{InsertOperation, SearchVector},
    engine::{
        index::hnsw::index::HNSWIndex,
        types::{types::VectorElementType, vector::VectorRef},
    },
};

#[utoipa::path(
    post,
    path = "/vector",
    request_body(
        content_type = "application/json",
        content = InsertOperation,
    ),
    responses(
        (status = 200, description = "Add Vectors in HSNW",)
    )
)]
#[post("/vector")]
pub async fn add_vector<'a>(
    data: Data<Arc<Mutex<HNSWIndex<'a>>>>,
    operation: web::Json<InsertOperation>,
) -> impl Responder {
    let points = operation.points.clone();
    match data.lock().unwrap().batch_add(points) {
        Ok(()) => "Vector added successfully",
        Err(e) => {
            log::error!("Error adding vector: {}", e);
            "Error adding vector"
        }
    }
}

#[utoipa::path(
    post,
    path = "/vector/search",
    responses(
        (status = 200, description = "Search Vectors in HSNW",)
    )
)]
#[post("/vector/search")]
pub async fn search_vector<'a>(
    data: Data<Arc<Mutex<HNSWIndex<'a>>>>,
    operation: Json<SearchVector>,
) -> impl Responder {
    let vector: &[VectorElementType] = operation.vector.as_slice();
    let top_k = operation.k;

    match data.lock().unwrap().search(vector, top_k) {
        Ok(result) => {
            let response = json!({
                "result": result,
            });
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Error searching vector: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/vector/dump")]
pub async fn dump_vector<'a>(data: Data<Arc<Mutex<HNSWIndex<'a>>>>) -> impl Responder {
    match data.lock().unwrap().dump() {
        Ok(result) => {
            let response = json!({
                "result": result,
            });
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Error searching vector: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
