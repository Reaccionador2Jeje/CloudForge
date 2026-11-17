mod config;
mod error;
mod instances;

use std::net::SocketAddr;

use axum::{routing::get, Json, Router};
use config::AppConfig;
use instances::{create_instance, list_instances, AppState};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    database: &'static str,
}

async fn live() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "cloudforge-api-gateway",
        database: "not_checked",
    })
}

async fn ready(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> (axum::http::StatusCode, Json<HealthResponse>) {
    let database_ready = match &state.pool {
        Some(pool) => pool.acquire().await.is_ok(),
        None => true, // Database is optional in the local API-only development mode.
    };

    let status = if database_ready {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(HealthResponse {
            status: if database_ready { "ready" } else { "degraded" },
            service: "cloudforge-api-gateway",
            database: if state.pool.is_some() {
                if database_ready {
                    "connected"
                } else {
                    "unavailable"
                }
            } else {
                "not_configured"
            },
        }),
    )
}

#[derive(Serialize)]
struct SystemInfo {
    name: &'static str,
    version: &'static str,
    stage: &'static str,
}

async fn system_info() -> Json<SystemInfo> {
    Json(SystemInfo {
        name: "CloudForge",
        version: env!("CARGO_PKG_VERSION"),
        stage: "development",
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a local .env file when present; process environment always takes precedence.
    let _ = dotenvy::dotenv();

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = AppConfig::from_env()?;
    let address: SocketAddr = config.socket_addr()?.parse()?;

    let pool = if let Some(database_url) = config.database_url.as_deref() {
        info!("Connecting to PostgreSQL");
        let pool = PgPoolOptions::new()
            .max_connections(config.database_max_connections)
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;
        info!("Database migrations completed");
        Some(pool)
    } else {
        info!("DATABASE_URL is not set; starting in API-only development mode");
        None
    };

    let state = AppState { pool };
    let app = Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/v1/system/info", get(system_info))
        .route("/v1/instances", get(list_instances).post(create_instance))
        .with_state(state);

    let listener = TcpListener::bind(address).await?;
    info!(address = %listener.local_addr()?, "CloudForge API Gateway is listening");

    axum::serve(listener, app)
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
