use std::net::SocketAddr;
use axum::{
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::trace::TraceLayer;
use tracing::info;

pub mod handlers;

use crate::state::RuleStore;
use handlers::{healthz_handler, mutate_handler, readyz_handler};

/// Constructs the Axum application router
pub fn build_app(store: RuleStore) -> Router {
    Router::new()
        .route("/healthz", get(healthz_handler))
        .route("/readyz", get(readyz_handler))
        .route("/mutate", post(mutate_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(store)
}

/// Runs the HTTPS server using axum-server and rustls
pub async fn run_tls_server(
    addr: SocketAddr,
    app: Router,
    tls_config: RustlsConfig,
) -> Result<(), std::io::Error> {
    info!(%addr, "Starting TLS webhook server");
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
}
