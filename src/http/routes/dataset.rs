use axum::{
    error_handling::HandleErrorLayer,
    extract::{
        DefaultBodyLimit,
        Multipart,
    },
    response::IntoResponse,
    routing::{
        self,
        get,
    },
    Extension,
    Json,
    Router,
};
use hyper::StatusCode;
use serde::{
    Deserialize,
    Serialize,
};
use tower::{
    Layer,
    ServiceBuilder,
};
use tower_http::limit::RequestBodyLimitLayer;
use utoipa::{
    openapi::request_body::RequestBodyBuilder,
    ToSchema,
};

use crate::http::errors::api_error::handle_error;

pub fn data_set_router() -> Router
{
    Router::new()
        .route(
            "/dataset",
            axum::routing::get(list_datasets).post(create_dataset),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(512 * 1024 * 1024))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024),
        )
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub enum DataType
{
    HDF5,
    CSV,
    PARQUET,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct DataSet
{
    #[serde(skip)]
    id: uuid::Uuid,
    name: String,
    description: String,
    datatype: DataType,
}


impl DataSet
{
    pub fn new(name: String, description: String, datatype: DataType) -> Self
    {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            description,
            datatype,
        }
    }
}

#[utoipa::path(
        get,
        path = "/dataset",
        responses(
            (status = 200, description = "List all DataSet successfully")
        )
 )]
pub async fn list_datasets() -> Json<Vec<DataSet>>
{
    let data_sets: Vec<DataSet> = vec![DataSet {
        id: uuid::Uuid::new_v4(),
        name: "name".to_string(),
        description: "description".to_string(),
        datatype: DataType::HDF5,
    }];
    Json(data_sets)
}

#[utoipa::path(
    post,
    path = "/dataset",
    request_body = Multipart,
    responses(
        (status = 200, description = "DataSet created successfully", body = String),
        (status = 400, description = "Bad Request"),
    )
)]
pub async fn create_dataset(mut multipart: Multipart) -> impl IntoResponse
{
    let file: Option<UploadedFile> = {
        let field = multipart.next_field().await;
        if let Ok(Some(field)) = field {
            let name = field.name().unwrap().to_string();
            let data = field.bytes().await.unwrap();
            Some(UploadedFile {
                filename: name,
                data,
            })
        } else {
            None
        }
    };
    if file.is_some() {
        return (StatusCode::OK, "DataSet created successfully")
    }
    return (StatusCode::BAD_REQUEST, "Bad Request")
}
