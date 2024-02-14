mod errors;
mod routes;
use axum::{routing::get, Router};
use std::io;
use tokio::{net::TcpListener, signal};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths())]
struct ApiDoc;

pub async fn init() -> io::Result<()> {
    let cors = CorsLayer::default().allow_origin(Any).allow_headers(Any).allow_methods(Any);
    let app = Router::new()
        .layer(cors)
        .route("/", get("hello from Qunatixar"))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listner = TcpListener::bind("0.0.0.0:8945").await.unwrap();
    info!(
        "Server is running on http://{}",
        listner.local_addr().unwrap()
    );
    axum::serve(listner, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
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
