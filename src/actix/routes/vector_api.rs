use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

use crate::actix::handlers::vector::{add_vector, search_vector};

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Welcome to the Quantixar API!",)
    )


)]
#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "message": "Welcome to the Quantixar API!"
    }))
}

pub fn config_index_api(cfg: &mut web::ServiceConfig) {
    cfg.service(index)
        .service(add_vector)
        .service(search_vector);
}
