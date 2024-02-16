use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::routes::index_api;

#[derive(OpenApi)]
#[openapi(paths(index_api::index))]
struct ApiDocs;

pub fn config_swagger_ui(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDocs::openapi()),
    );
}
