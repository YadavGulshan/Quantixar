pub mod handlers;
pub mod model;
pub mod routes;
pub mod table;
use std::{
    io::Error,
    net::{Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use actix_cors::Cors;
use actix_multipart::form::Limits;
use actix_web::{
    error, get,
    middleware::{Compress, Logger},
    web::{self, Data, PayloadConfig},
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
        512,
        crate::engine::types::distance::Distance::Cosine,
        "quantixar",
    );
    let vector_storage = Arc::new(AtomicRefCell::new(VectorStorageEnum::DenseSimple(
        simple_dense_vector_stroage,
    )));

    let index_dir = "index_dir";
    let index_dir_path = std::path::Path::new(index_dir);
    let data_dimension = 512;
    let dataset_size = 10;
    let dist_f = hnsw_rs::dist::DistCosine;
    let engine = match HNSWIndex::new(
        vector_storage,
        index_dir_path,
        data_dimension,
        dataset_size,
        dist_f,
    ) {
        Ok(mut engine) => match engine.build_graph(true) {
            Ok(_) => Arc::new(Mutex::new(engine)),
            Err(e) => panic!("Error building HNSWIndex: {}", e),
        },
        Err(e) => panic!("Error creating HNSWIndex: {}", e),
    };
    // custom `Json` extractor configuration
    let server = HttpServer::new(move || {
        let json_cfg = web::JsonConfig::default()
            // limit request payload size
            .limit(1000000 * 25)
            // only accept text/plain content type
            // .content_type(|mime| mime == mime::TEXT_PLAIN)
            // use custom error handler
            .error_handler(|err, req| {
                error::InternalError::from_response(err, HttpResponse::Conflict().into()).into()
            });
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
