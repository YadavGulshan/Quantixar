use std::{
    convert::Infallible,
    io::{Error, Read, Write},
    path::Path,
};

use actix_multipart::{
    form::{tempfile::TempFile, MultipartForm},
    Multipart,
};
use actix_web::{http::StatusCode, post, HttpResponse, Responder};
use chrono::format;
use serde::{Deserialize, Serialize};
use tracing::debug;
use utoipa::{openapi::Components, ToSchema};

use crate::common::operation_error::OperationResult;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub enum DataType {
    // HDF5,
    CSV,
    PARQUET,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct DataSet {
    #[serde(skip)]
    id: uuid::Uuid,
    name: String,
    description: String,
    datatype: DataType,
}

impl DataSet {
    pub fn new(name: String, description: String, datatype: DataType) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            description,
            datatype,
        }
    }
}

#[derive(Debug, ToSchema)]
pub struct UploadedFileSw {
    #[schema(format = Binary)]
    file: Vec<u8>,
}

#[derive(Debug, MultipartForm, ToSchema)]
pub struct UploadedFile {
    #[multipart(rename = "file")]
    #[schema(format = Binary)]
    file: TempFile,
}

#[utoipa::path(
post,
path = "/dataset",
request_body(
content_type = "multipart/form-data",
content = UploadedFileSw
),
responses(
(status = 200, description = "DataSet created successfully")
)
)]
#[post("/dataset")]
pub async fn create_dataset(
    MultipartForm(form): MultipartForm<UploadedFile>,
) -> OperationResult<HttpResponse> {
    Ok(HttpResponse::Ok().body("DataSet created successfully"))
}

pub fn config_dataset_api(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(create_dataset);
}
