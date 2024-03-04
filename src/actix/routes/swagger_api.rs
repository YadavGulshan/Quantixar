use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::actix;
use crate::actix::handlers::vector;

use crate::actix::routes::{dataset_api, vector_api};
#[derive(OpenApi)]
#[openapi(
    paths(
        vector_api::index,
        dataset_api::create_dataset,
        vector::add_vector,
        vector::search_vector
    ),
    components(schemas(
        dataset_api::UploadedFileSw,
        actix::model::vector::InsertOperation,
        actix::model::vector::SearchVector,
        actix::model::vector::AddVector,
    ))
)]
struct ApiDocs;

pub fn config_swagger_ui(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDocs::openapi()),
    );
}
