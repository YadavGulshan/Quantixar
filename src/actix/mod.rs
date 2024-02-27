pub mod handlers;
mod model;
pub mod routes;
pub mod table;
use std::{
    io::Error,
    net::{Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use actix_cors::Cors;
use actix_web::{
    get,
    middleware::{Compress, Logger},
    web::Data,
    App, HttpResponse, HttpServer, Responder,
};
use atomic_refcell::AtomicRefCell;
use routes::dataset_api;
use serde_json::json;
use tokio::{net::TcpListener, signal};
use tracing::info;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    actix::{
        handlers::vector,
        routes::{
            dataset_api::config_dataset_api, swagger_api::config_swagger_ui,
            vector_api::config_index_api,
        },
    },
    engine::{
        index::hnsw::index::HNSWIndex,
        storage::vector::{
            base::VectorStorageEnum, dense_vector_storage::SimpleDenseVectorStorage,
        },
    },
    setting::Settings,
};

pub async fn init(settings: Settings) -> Result<(), Error> {
    let simple_dense_vector_stroage = SimpleDenseVectorStorage::new(
        5,
        crate::engine::types::distance::Distance::Euclidean,
        "quantixar",
    );
    let vector_storage = Arc::new(AtomicRefCell::new(VectorStorageEnum::DenseSimple(
        simple_dense_vector_stroage,
    )));

    let path_to_rocks_db = "rockdb";
    let path = std::path::Path::new(path_to_rocks_db);
    let data_dimension = 5;
    let dataset_size = 10;
    let dist_f = hnsw_rs::dist::DistL2;
    let engine = match HNSWIndex::new(vector_storage, path, data_dimension, dataset_size, dist_f) {
        Ok(engine) => Arc::new(Mutex::new(engine)),
        Err(e) => panic!("Error creating HNSWIndex: {}", e),
    };
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .allow_any_origin();
        App::new()
            .wrap(Compress::default())
            .wrap(cors)
            .wrap(Logger::default().exclude("/"))
            .app_data(Data::new(engine.clone()))
            .configure(config_index_api)
            .configure(config_swagger_ui)
            .configure(config_dataset_api)
    });
    let port = settings.service.http_port;
    let host = settings.service.host;
    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port);
    info!("Starting server at http://{}", addr);
    server.bind(addr)?.run().await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
