use std::{
  io::{Error, Read, Write},
  path::Path,
};
use std::convert::Infallible;
use std::ops::FromResidual;

use actix_multipart::{
  form::{MultipartForm, tempfile::TempFile},
  Multipart,
};
use actix_web::{http::StatusCode, HttpResponse, post, Responder};
use chrono::format;
use hdf5::{File as Hdf5File, H5Type, Result};
use serde::{Deserialize, Serialize};
use tower::{Layer, ServiceBuilder};
use tower_http::limit::RequestBodyLimitLayer;
use tracing::debug;
use utoipa::{openapi::Components, ToSchema};

use crate::common::operation_error::OperationResult;
use crate::http::handlers::dataset::read_hdf5;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub enum DataType {
  HDF5,
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

pub struct HDF5FileConfig<'a> {
  pub dataset_name: &'a str,
  pub target_data_path: &'a str,
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
  let file = form.file;
  let file_path = file.file.path();
  debug!("File path: {:?}", file_path);
  read_hdf5(
    &file_path.to_str().ok_or(Error::new(
      std::io::ErrorKind::InvalidData,
      "Invalid file path",
    ))?,
    HDF5FileConfig {
      dataset_name: "data",
      target_data_path: "test",
    },
  )?;
  Ok(HttpResponse::Ok().body("DataSet created successfully"))
}

pub fn config_dataset_api(cfg: &mut actix_web::web::ServiceConfig) {
  cfg.service(create_dataset);
}
