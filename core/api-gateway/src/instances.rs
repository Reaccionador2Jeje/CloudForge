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
    pub vcpu: i32,
    pub memory_mib: i32,
    pub image_ref: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInstanceRequest {
    pub name: String,
    /// Requested virtual CPU count. Defaults to one.
    pub vcpu: Option<i32>,
    /// Requested memory in MiB. Defaults to 1024 MiB.
    pub memory_mib: Option<i32>,
    /// Optional future catalog image identifier; no image is downloaded or booted.
    pub image_ref: Option<String>,
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

fn validate_compute_spec(
    vcpu: Option<i32>,
    memory_mib: Option<i32>,
    image_ref: Option<&str>,
) -> Result<(i32, i32, Option<String>), ApiError> {
    let vcpu = vcpu.unwrap_or(1);
    let memory_mib = memory_mib.unwrap_or(1024);

    if !(1..=64).contains(&vcpu) {
        return Err(ApiError::Validation(
            "vcpu must be between 1 and 64".to_owned(),
        ));
    }
    if !(256..=262_144).contains(&memory_mib) {
        return Err(ApiError::Validation(
            "memory_mib must be between 256 and 262144".to_owned(),
        ));
    }

    let image_ref = image_ref
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);

    if let Some(reference) = image_ref.as_deref() {
        if reference.len() > 128
            || !reference.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, '.' | '_' | ':' | '/' | '-')
            })
            || !reference
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphanumeric())
        {
            return Err(ApiError::Validation(
                "image_ref must be a valid catalog-style identifier of at most 128 characters"
                    .to_owned(),
            ));
        }
    }

    Ok((vcpu, memory_mib, image_ref))
}

pub async fn list_instances(
    State(state): State<AppState>,
) -> Result<Json<Vec<Instance>>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let instances = sqlx::query_as::<_, Instance>(
        "SELECT id, name, status, vcpu, memory_mib, image_ref, created_at
         FROM instances ORDER BY created_at DESC",
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
    let (vcpu, memory_mib, image_ref) = validate_compute_spec(
        request.vcpu,
        request.memory_mib,
        request.image_ref.as_deref(),
    )?;
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let instance = sqlx::query_as::<_, Instance>(
        "INSERT INTO instances (name, vcpu, memory_mib, image_ref)
         VALUES ($1, $2, $3, $4)
         RETURNING id, name, status, vcpu, memory_mib, image_ref, created_at",
    )
    .bind(name)
    .bind(vcpu)
    .bind(memory_mib)
    .bind(image_ref)
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

    #[test]
    fn compute_spec_defaults_are_safe_and_predictable() {
        assert_eq!(
            validate_compute_spec(None, None, None).unwrap(),
            (1, 1024, None)
        );
    }

    #[test]
    fn accepts_valid_compute_spec_and_normalizes_image_ref() {
        assert_eq!(
            validate_compute_spec(Some(8), Some(8192), Some(" ubuntu/24.04 ")).unwrap(),
            (8, 8192, Some("ubuntu/24.04".to_owned()))
        );
    }

    #[test]
    fn rejects_compute_spec_outside_supported_limits() {
        assert!(validate_compute_spec(Some(0), Some(1024), None).is_err());
        assert!(validate_compute_spec(Some(65), Some(1024), None).is_err());
        assert!(validate_compute_spec(Some(2), Some(255), None).is_err());
        assert!(validate_compute_spec(Some(2), Some(262_145), None).is_err());
    }

    #[test]
    fn rejects_invalid_image_references() {
        assert!(validate_compute_spec(Some(2), Some(2048), Some("../private")).is_err());
        assert!(validate_compute_spec(Some(2), Some(2048), Some("ubuntu image")).is_err());
        assert!(validate_compute_spec(Some(2), Some(2048), Some(&"a".repeat(129))).is_err());
    }
}
