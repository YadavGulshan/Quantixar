use std::{
    io::{Error, Read, Write},
    path::Path,
};

use crate::http::handlers::dataset::read_hdf5;
use actix_multipart::{
    form::{tempfile::TempFile, MultipartForm},
    Multipart,
};
use actix_web::{http::StatusCode, post, HttpResponse, Responder};

use chrono::format;
use hdf5::{File as Hdf5File, H5Type, Result};
use serde::{Deserialize, Serialize};
use tower::{Layer, ServiceBuilder};
use tower_http::limit::RequestBodyLimitLayer;
use tracing::debug;
use utoipa::{openapi::Components, ToSchema};

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

// #[utoipa::path(
//         get,
//         path = "/dataset",
//         responses(
//             (status = 200, description = "List all DataSet successfully")
//         )
//  )]
// pub async fn list_datasets() -> Json<Vec<DataSet>> {
//     let data_sets: Vec<DataSet> = vec![DataSet {
//         id: uuid::Uuid::new_v4(),
//         name: "name".to_string(),
//         description: "description".to_string(),
//         datatype: DataType::HDF5,
//     }];
//     Json(data_sets)
// }

#[derive(Debug, ToSchema)]
pub struct UploadedFile_SW {
    #[schema( format = Binary)]
    file: Vec<u8>,
}

#[derive(Debug, MultipartForm, ToSchema)]
pub struct UploadedFile {
    #[multipart(rename = "file")]
    #[schema( format = Binary)]
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
        content = UploadedFile_SW
    ),
    responses(
        (status = 200, description = "DataSet created successfully")
    )
)]
#[post("/dataset")]
pub async fn create_dataset(
    MultipartForm(form): MultipartForm<UploadedFile>,
) -> Result<HttpResponse, Error> {
    let file = form.file;
    let file_path = file.file.path();
    debug!("File path: {:?}", file_path);
    read_hdf5(
        &file_path.to_str().unwrap(),
        HDF5FileConfig {
            dataset_name: "data",
            target_data_path: "test",
        },
    )
    .unwrap();
    Ok(HttpResponse::Ok().body("DataSet created successfully"))
}

pub fn config_dataset_api(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(create_dataset);
}
