mod auth;
mod components;
mod compute;
mod config;
mod error;
mod guest_profiles;
mod images;
mod installations;
mod instances;
mod nodes;
mod storage;

use std::net::SocketAddr;

use axum::{
    middleware,
    response::Html,
    routing::{delete, get, post},
    Json, Router,
};
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

async fn web_console() -> Html<&'static str> {
    Html(include_str!("../../../web/console/index.html"))
}

async fn web_styles() -> impl axum::response::IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../../../web/console/styles.css"),
    )
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
    let instance_gate = auth::PermissionGate {
        state: state.clone(),
        resource: "instances",
    };
    let installation_gate = auth::PermissionGate {
        state: state.clone(),
        resource: "installations",
    };
    let image_gate = auth::PermissionGate {
        state: state.clone(),
        resource: "images",
    };
    let node_gate = auth::PermissionGate {
        state: state.clone(),
        resource: "nodes",
    };
    let storage_gate = auth::PermissionGate {
        state: state.clone(),
        resource: "storage",
    };
    let app =
        Router::new()
            .route("/", get(web_console))
            .route("/console", get(web_console))
            .route("/styles.css", get(web_styles))
            .route("/health/live", get(live))
            .route("/health/ready", get(ready))
            .route("/v1/system/info", get(system_info))
            .route("/v1/auth/register", post(auth::register))
            .route("/v1/auth/bootstrap", post(auth::bootstrap))
            .route("/v1/auth/login", post(auth::login))
            .route(
                "/v1/auth/me",
                get(auth::me).route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth::require_session,
                )),
            )
            .route(
                "/v1/auth/logout",
                post(auth::logout).route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth::require_session,
                )),
            )
            .route("/v1/system/components", get(components::list_components))
            .route("/v1/compute/providers", get(compute::list_providers))
            .route(
                "/v1/compute/guest-profiles",
                get(guest_profiles::list_guest_profiles),
            )
            .route(
                "/v1/compute/installations",
                get(installations::list_installations)
                    .post(installations::create_installation)
                    .route_layer(middleware::from_fn_with_state(
                        installation_gate.clone(),
                        auth::require_permission,
                    )),
            )
            .route(
                "/v1/compute/installations/{id}",
                get(installations::get_installation).route_layer(middleware::from_fn_with_state(
                    installation_gate,
                    auth::require_permission,
                )),
            )
            .route(
                "/v1/images",
                get(images::list_images)
                    .post(images::create_image)
                    .route_layer(middleware::from_fn_with_state(
                        image_gate,
                        auth::require_permission,
                    )),
            )
            .route(
                "/v1/instances",
                get(list_instances).post(create_instance).route_layer(
                    middleware::from_fn_with_state(instance_gate, auth::require_permission),
                ),
            )
            .route(
                "/v1/nodes",
                get(nodes::list_nodes).route_layer(middleware::from_fn_with_state(
                    node_gate,
                    auth::require_permission,
                )),
            )
            .route(
                "/v1/storage/quota",
                get(storage::get_quota).route_layer(middleware::from_fn_with_state(
                    storage_gate.clone(),
                    auth::require_permission,
                )),
            )
            .route(
                "/v1/storage/reservations",
                get(storage::list_reservations)
                    .post(storage::create_reservation)
                    .route_layer(middleware::from_fn_with_state(
                        storage_gate.clone(),
                        auth::require_permission,
                    )),
            )
            .route(
                "/v1/storage/reservations/{id}",
                delete(storage::release_reservation).route_layer(middleware::from_fn_with_state(
                    storage_gate.clone(),
                    auth::require_permission,
                )),
            )
            .route(
                "/v1/storage/volumes",
                get(storage::list_volumes)
                    .post(storage::create_volume)
                    .route_layer(middleware::from_fn_with_state(
                        storage_gate.clone(),
                        auth::require_permission,
                    )),
            )
            .route(
                "/v1/storage/volumes/{id}",
                delete(storage::delete_volume).route_layer(middleware::from_fn_with_state(
                    storage_gate,
                    auth::require_permission,
                )),
            )
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
