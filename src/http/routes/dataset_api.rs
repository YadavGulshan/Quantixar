use std::path::Path;

use axum::{
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, Multipart},
    response::{Html, IntoResponse},
    routing::{self, get},
    Extension, Json, Router,
};
use chrono::format;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tower::{Layer, ServiceBuilder};
use tower_http::limit::RequestBodyLimitLayer;
use utoipa::{openapi::request_body::RequestBodyBuilder, ToSchema};

use crate::http::errors::api_error::handle_error;
use hdf5::{File, H5Type, Result};

pub fn data_set_router() -> Router {
    Router::new()
        .route("/", axum::routing::get(show_form).post(create_dataset))
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

#[utoipa::path(
        get,
        path = "/dataset",
        responses(
            (status = 200, description = "List all DataSet successfully")
        )
 )]
pub async fn list_datasets() -> Json<Vec<DataSet>> {
    let data_sets: Vec<DataSet> = vec![DataSet {
        id: uuid::Uuid::new_v4(),
        name: "name".to_string(),
        description: "description".to_string(),
        datatype: DataType::HDF5,
    }];
    Json(data_sets)
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UploadedFile {
    pub filename: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FileUpload {
    file: UploadedFile,
}

pub struct HDF5FileConfig<'a> {
    pub dataset_name: &'a str,
    pub target_data_path: &'a str,
}

#[utoipa::path(
    post,
    path = "/",
    request_body = Multipart,
    responses(
        (status = 200, description = "DataSet created successfully", body = String),
        (status = 400, description = "Bad Request"),
    )
)]
pub async fn create_dataset(mut multipart: Multipart) -> impl IntoResponse {
    let file: Option<UploadedFile> = {
        let field = multipart.next_field().await;
        if let Ok(Some(field)) = field {
            let name = field.name().unwrap().to_string();
            let data = field.bytes().await.unwrap();
            Some(UploadedFile {
                filename: name,
                data: data.to_vec(),
            })
        } else {
            None
        }
    };
    if file.is_none() {
        return (StatusCode::BAD_REQUEST, "Bad Request");
    }
    // Save data to /tmp
    let file = file.unwrap();
    let file_path = format!("/tmp/{}", file.filename);
    std::fs::write(file_path.as_str(), file.data).unwrap();

    read_hdf5(
        &file_path,
        HDF5FileConfig {
            dataset_name: "data",
            target_data_path: "test",
        },
    )
    .unwrap();

    (StatusCode::OK, "DataSet created successfully")
}

pub fn read_hdf5(file_path: &str, config: HDF5FileConfig) -> Result<()> {
    let file = File::open(file_path)?;
    let dataset = file.dataset(config.target_data_path)?;
    println!("{:?}", dataset.read_raw::<f32>()?);
    // let attr = dataset.attr("/test")?;
    // println!("{:?}", attr.read_1d::<f32>()?.as_slice());
    Ok(())
}

async fn show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/" method="post" enctype="multipart/form-data">
                    <label>
                        Upload file:
                        <input type="file" name="file" multiple>
                    </label>

                    <input type="submit" value="Upload files">
                </form>
            </body>
        </html>
        "#,
    )
}
