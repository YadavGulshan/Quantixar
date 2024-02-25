use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::actix::routes::{dataset_api, vector};
#[derive(OpenApi)]
#[openapi(
    paths(vector::index, dataset_api::create_dataset),
    components(schemas(dataset_api::UploadedFileSw))
)]
struct ApiDocs;

pub fn config_swagger_ui(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDocs::openapi()),
    );
}
