pub mod errors;
pub mod hanlders;
pub mod routes;

use std::{
    io::Error,
    net::{
        Ipv4Addr,
        SocketAddr,
    },
};

use axum::Router;
use routes::dataset;
use tokio::{
    net::TcpListener,
    signal,
};
use tower_http::cors::{
    Any,
    CorsLayer,
};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{
    Redoc,
    Servable,
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(dataset::list_datasets, dataset::create_dataset),
    components(schemas(dataset::DataSet), schemas(dataset::FileUpload))
)]
struct ApiDoc;

pub async fn init() -> Result<(), Error>
{
    let cors = CorsLayer::default().allow_origin(Any).allow_headers(Any).allow_methods(Any);
    let dataset_api = routes::dataset::data_set_router();
    let app = Router::new()
        .layer(cors)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .nest("/", dataset_api);

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8945));
    let listener = TcpListener::bind(&address).await?;
    log::info!("Listening on {}", address);
    axum::serve(listener, app.into_make_service()).await
}

async fn shutdown_signal()
{
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
