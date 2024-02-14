use axum::{extract::rejection::JsonRejection, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    JsonExtractionRejection(#[from] JsonRejection),
    #[error("{0}")]
    BadRequest(String),
    #[error("")]
    NotFound(String),
    #[error("Internal Server Error")]
    InternalServerError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::JsonExtractionRejection(json_rejection) => {
                (json_rejection.status(), json_rejection.body_text())
            }
            ApiError::BadRequest(message) => (axum::http::StatusCode::BAD_REQUEST, message),
            ApiError::NotFound(message) => (axum::http::StatusCode::NOT_FOUND, message),
            ApiError::InternalServerError => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
        };

        let payload = json!({
            "message": message,
            "status": status.as_u16(),
        });
        tracing::error!("Error: {}", message);
        (status, Json(payload)).into_response()
    }
}
