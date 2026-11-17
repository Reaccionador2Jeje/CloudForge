use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Clone)]
pub struct AppState {
    pub pool: Option<PgPool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Instance {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInstanceRequest {
    pub name: String,
}

fn validate_instance_name(name: &str) -> Result<&str, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::Validation("name cannot be empty".to_owned()));
    }
    if name.len() > 63 {
        return Err(ApiError::Validation(
            "name cannot exceed 63 UTF-8 bytes".to_owned(),
        ));
    }
    Ok(name)
}

pub async fn list_instances(
    State(state): State<AppState>,
) -> Result<Json<Vec<Instance>>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let instances = sqlx::query_as::<_, Instance>(
        "SELECT id, name, status, created_at FROM instances ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(Json(instances))
}

pub async fn create_instance(
    State(state): State<AppState>,
    Json(request): Json<CreateInstanceRequest>,
) -> Result<(StatusCode, Json<Instance>), ApiError> {
    let name = validate_instance_name(&request.name)?;
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let instance = sqlx::query_as::<_, Instance>(
        "INSERT INTO instances (name) VALUES ($1) RETURNING id, name, status, created_at",
    )
    .bind(name)
    .fetch_one(pool)
    .await?;

    Ok((StatusCode::CREATED, Json(instance)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_instance_names() {
        assert_eq!(
            validate_instance_name("  test-node  ").unwrap(),
            "test-node"
        );
    }

    #[test]
    fn rejects_empty_instance_names() {
        assert!(validate_instance_name("   ").is_err());
    }

    #[test]
    fn rejects_names_longer_than_63_bytes() {
        assert!(validate_instance_name(&"a".repeat(64)).is_err());
    }

    #[test]
    fn accepts_63_byte_names() {
        assert!(validate_instance_name(&"a".repeat(63)).is_ok());
    }
}
