mod model;
pub mod handlers;
pub mod routes;
pub mod table;
use std::{
    io::Error,
    net::{Ipv4Addr, SocketAddr},
};

use actix_cors::Cors;
use actix_web::{
    get,
    middleware::{Compress, Logger},
    App, HttpResponse, HttpServer, Responder,
};
use routes::dataset_api;
use serde_json::json;
use tokio::{net::TcpListener, signal};
use tracing::info;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    http::routes::{
        dataset_api::config_dataset_api, index_api::config_index_api,
        swagger_api::config_swagger_ui,
    },
    setting::Settings,
};

pub async fn init(settings: Settings) -> Result<(), Error> {
    let server = HttpServer::new(move || {
        let cors = Cors::default().allow_any_header().allow_any_method().allow_any_origin();
        App::new()
            .wrap(Compress::default())
            .wrap(cors)
            .wrap(Logger::default().exclude("/"))
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
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
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
