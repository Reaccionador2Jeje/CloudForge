use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{error::ApiError, instances::AppState};

/// Public-safe node registry view. Certificate fingerprints are never returned.
#[derive(Debug, Serialize, FromRow)]
pub struct NodeSummary {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub agent_version: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Lists registered nodes for the local control-plane console.
/// This is read-only: it cannot enroll, activate, or revoke nodes.
pub async fn list_nodes(
    State(state): State<AppState>,
) -> Result<Json<Vec<NodeSummary>>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let nodes = sqlx::query_as::<_, NodeSummary>(
        "SELECT id, name, status, agent_version, last_seen_at, created_at
         FROM nodes ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(Json(nodes))
}
