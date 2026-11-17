use std::{env, net::SocketAddr};

use axum::{
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Clone, Debug)]
struct AppConfig {
    host: String,
    port: u16,
}

impl AppConfig {
    fn from_env() -> Result<Self, String> {
        let host = env::var("CLOUDFORGE_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_owned());

        let port = env::var("CLOUDFORGE_PORT")
            .unwrap_or_else(|_| "8080".to_owned())
            .parse::<u16>()
            .map_err(|_| "CLOUDFORGE_PORT must be a valid TCP port".to_owned())?;

        if host.trim().is_empty() {
            return Err("CLOUDFORGE_HOST cannot be empty".to_owned());
        }

        Ok(Self { host, port })
    }

    fn socket_addr(&self) -> Result<SocketAddr, String> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|error| format!("Invalid listen address: {error}"))
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn live() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "cloudforge-api-gateway",
    })
}

async fn ready() -> (StatusCode, Json<HealthResponse>) {
    // This initial readiness endpoint only confirms that the HTTP service is running.
    // Dependency checks will be added when the database and internal services are integrated.
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ready",
            service: "cloudforge-api-gateway",
        }),
    )
}

fn app() -> Router {
    Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    let config = AppConfig::from_env()?;
    let address = config.socket_addr()?;
    let listener = TcpListener::bind(address).await?;

    info!(
        address = %listener.local_addr()?,
        "CloudForge API Gateway is listening"
    );

    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "Failed to listen for shutdown signal");
    }

    info!("Shutdown signal received");
}
