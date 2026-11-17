use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::AuthenticatedUser, error::ApiError, instances::AppState};

const RESERVATION_TTL_MINUTES: i64 = 30;
const MAX_IDEMPOTENCY_KEY_BYTES: usize = 128;

#[derive(Debug, Serialize, FromRow)]
pub struct StorageReservation {
    pub id: Uuid,
    pub idempotency_key: String,
    pub requested_gib: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub released_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStorageReservationRequest {
    pub requested_gib: i64,
}

#[derive(Debug, Serialize)]
pub struct StorageQuotaView {
    pub quota_gib: i64,
    pub used_gib: i64,
    pub reserved_gib: i64,
    pub available_gib: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StorageVolume {
    pub id: Uuid,
    pub reservation_id: Option<Uuid>,
    pub name: String,
    pub size_gib: i64,
    pub format: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStorageVolumeRequest {
    pub reservation_id: Uuid,
    pub name: String,
    pub format: String,
}

fn validate_idempotency_key(value: &str) -> Result<&str, ApiError> {
    let value = value.trim();
    if !(8..=MAX_IDEMPOTENCY_KEY_BYTES).contains(&value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(ApiError::Validation(
            "Idempotency-Key must contain 8-128 ASCII letters, digits, or -_.: characters"
                .to_owned(),
        ));
    }
    Ok(value)
}

fn validate_volume_name(value: &str) -> Result<&str, ApiError> {
    if value.is_empty()
        || value.len() > 48
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
    {
        return Err(ApiError::Validation(
            "Volume name must start with a lowercase letter and contain at most 48 lowercase letters, digits, hyphens, or underscores".to_owned(),
        ));
    }
    Ok(value)
}

async fn ensure_quota_row(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<i64, ApiError> {
    sqlx::query(
        "INSERT INTO storage_quotas (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(user_id)
    .execute(&mut **transaction)
    .await?;

    let quota: i64 =
        sqlx::query_scalar("SELECT quota_gib FROM storage_quotas WHERE user_id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_one(&mut **transaction)
            .await?;
    Ok(quota)
}

pub async fn get_quota(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<StorageQuotaView>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let quota: i64 = sqlx::query_scalar("SELECT quota_gib FROM storage_quotas WHERE user_id = $1")
        .bind(user.id)
        .fetch_optional(pool)
        .await?
        .unwrap_or(100);

    let used_gib: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(size_gib), 0)::BIGINT FROM storage_volumes
         WHERE user_id = $1 AND status IN ('planned', 'provisioning', 'ready', 'deleting')",
    )
    .bind(user.id)
    .fetch_one(pool)
    .await?;

    let reserved_gib: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(requested_gib), 0)::BIGINT FROM storage_reservations
         WHERE user_id = $1 AND status = 'reserved' AND expires_at > NOW()",
    )
    .bind(user.id)
    .fetch_one(pool)
    .await?;

    Ok(Json(StorageQuotaView {
        quota_gib: quota,
        used_gib,
        reserved_gib,
        available_gib: quota.saturating_sub(used_gib.saturating_add(reserved_gib)),
    }))
}

pub async fn list_reservations(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<StorageReservation>>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let reservations = sqlx::query_as::<_, StorageReservation>(
        "SELECT id, idempotency_key, requested_gib, status, created_at, expires_at, released_at
         FROM storage_reservations
         WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT 100",
    )
    .bind(user.id)
    .fetch_all(pool)
    .await?;
    Ok(Json(reservations))
}

pub async fn create_reservation(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CreateStorageReservationRequest>,
) -> Result<(StatusCode, Json<StorageReservation>), ApiError> {
    if !(1..=2048).contains(&request.requested_gib) {
        return Err(ApiError::Validation(
            "requested_gib must be between 1 and 2048".to_owned(),
        ));
    }
    let key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::Validation("Idempotency-Key header is required".to_owned()))?;
    let key = validate_idempotency_key(key)?;

    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let mut transaction = pool.begin().await?;
    let quota = ensure_quota_row(&mut transaction, user.id).await?;

    if let Some(existing) = sqlx::query_as::<_, StorageReservation>(
        "SELECT id, idempotency_key, requested_gib, status, created_at, expires_at, released_at
         FROM storage_reservations WHERE user_id = $1 AND idempotency_key = $2",
    )
    .bind(user.id)
    .bind(key)
    .fetch_optional(&mut *transaction)
    .await?
    {
        if existing.requested_gib != request.requested_gib
            || existing.status != "reserved"
            || existing.expires_at <= Utc::now()
        {
            return Err(ApiError::Conflict(
                "Idempotency-Key was already used for a different or inactive reservation"
                    .to_owned(),
            ));
        }
        transaction.commit().await?;
        return Ok((StatusCode::OK, Json(existing)));
    }

    let used_gib: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(size_gib), 0)::BIGINT FROM storage_volumes
         WHERE user_id = $1 AND status IN ('planned', 'provisioning', 'ready', 'deleting')",
    )
    .bind(user.id)
    .fetch_one(&mut *transaction)
    .await?;

    let reserved_gib: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(requested_gib), 0)::BIGINT FROM storage_reservations
         WHERE user_id = $1 AND status = 'reserved' AND expires_at > NOW()",
    )
    .bind(user.id)
    .fetch_one(&mut *transaction)
    .await?;

    let projected = used_gib
        .saturating_add(reserved_gib)
        .saturating_add(request.requested_gib);
    if projected > quota {
        return Err(ApiError::Conflict(format!(
            "Storage quota exceeded: {projected} GiB requested against a {quota} GiB quota"
        )));
    }

    let reservation = sqlx::query_as::<_, StorageReservation>(
        "INSERT INTO storage_reservations
            (user_id, idempotency_key, requested_gib, expires_at)
         VALUES ($1, $2, $3, NOW() + ($4 * INTERVAL '1 minute'))
         RETURNING id, idempotency_key, requested_gib, status, created_at, expires_at, released_at",
    )
    .bind(user.id)
    .bind(key)
    .bind(request.requested_gib)
    .bind(RESERVATION_TTL_MINUTES)
    .fetch_one(&mut *transaction)
    .await?;

    sqlx::query(
        "INSERT INTO audit_events
            (actor_user_id, event_type, resource_type, resource_id, outcome, details)
         VALUES ($1, 'storage.reservation.created', 'storage_reservation', $2, 'success',
                 jsonb_build_object('requested_gib', $3, 'idempotency_key', $4))",
    )
    .bind(user.id)
    .bind(reservation.id.to_string())
    .bind(request.requested_gib)
    .bind(key)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json(reservation)))
}

pub async fn release_reservation(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let mut transaction = pool.begin().await?;
    let _quota = ensure_quota_row(&mut transaction, user.id).await?;
    let reservation = sqlx::query_as::<_, StorageReservation>(
        "SELECT id, idempotency_key, requested_gib, status, created_at, expires_at, released_at
         FROM storage_reservations
         WHERE id = $1 AND user_id = $2
         FOR UPDATE",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ApiError::NotFound("storage reservation was not found".to_owned()))?;

    if reservation.status == "reserved" {
        sqlx::query(
            "UPDATE storage_reservations
             SET status = 'released', released_at = NOW()
             WHERE id = $1",
        )
        .bind(id)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "INSERT INTO audit_events
                (actor_user_id, event_type, resource_type, resource_id, outcome)
             VALUES ($1, 'storage.reservation.released', 'storage_reservation', $2, 'success')",
        )
        .bind(user.id)
        .bind(id.to_string())
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_volumes(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<StorageVolume>>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let volumes = sqlx::query_as::<_, StorageVolume>(
        "SELECT id, reservation_id, name, size_gib, format, status, created_at, updated_at
         FROM storage_volumes WHERE user_id = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(user.id)
    .fetch_all(pool)
    .await?;
    Ok(Json(volumes))
}

pub async fn create_volume(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateStorageVolumeRequest>,
) -> Result<(StatusCode, Json<StorageVolume>), ApiError> {
    let name = validate_volume_name(&request.name)?;
    if !matches!(request.format.as_str(), "qcow2" | "raw") {
        return Err(ApiError::Validation(
            "format must be qcow2 or raw".to_owned(),
        ));
    }

    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let mut transaction = pool.begin().await?;
    let _quota = ensure_quota_row(&mut transaction, user.id).await?;

    let reservation = sqlx::query_as::<_, StorageReservation>(
        "SELECT id, idempotency_key, requested_gib, status, created_at, expires_at, released_at
         FROM storage_reservations
         WHERE id = $1 AND user_id = $2 FOR UPDATE",
    )
    .bind(request.reservation_id)
    .bind(user.id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ApiError::NotFound("active storage reservation was not found".to_owned()))?;

    if reservation.status == "consumed" {
        let existing = sqlx::query_as::<_, StorageVolume>(
            "SELECT id, reservation_id, name, size_gib, format, status, created_at, updated_at
             FROM storage_volumes WHERE reservation_id = $1 AND user_id = $2",
        )
        .bind(reservation.id)
        .bind(user.id)
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(existing) = existing {
            if existing.name == name && existing.format == request.format {
                transaction.commit().await?;
                return Ok((StatusCode::OK, Json(existing)));
            }
            return Err(ApiError::Conflict(
                "This reservation has already been used to create a different volume".to_owned(),
            ));
        }
    }

    if reservation.status != "reserved" || reservation.expires_at <= Utc::now() {
        return Err(ApiError::Conflict(
            "Storage reservation is no longer active".to_owned(),
        ));
    }

    let name_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM storage_volumes WHERE user_id = $1 AND name = $2)",
    )
    .bind(user.id)
    .bind(name)
    .fetch_one(&mut *transaction)
    .await?;
    if name_exists {
        return Err(ApiError::Conflict(
            "A volume with this name already exists".to_owned(),
        ));
    }

    let volume = sqlx::query_as::<_, StorageVolume>(
        "INSERT INTO storage_volumes (user_id, reservation_id, name, size_gib, format, status)
         VALUES ($1, $2, $3, $4, $5, 'planned')
         RETURNING id, reservation_id, name, size_gib, format, status, created_at, updated_at",
    )
    .bind(user.id)
    .bind(reservation.id)
    .bind(name)
    .bind(reservation.requested_gib)
    .bind(&request.format)
    .fetch_one(&mut *transaction)
    .await?;

    sqlx::query("UPDATE storage_reservations SET status = 'consumed' WHERE id = $1")
        .bind(reservation.id)
        .execute(&mut *transaction)
        .await?;

    sqlx::query(
        "INSERT INTO audit_events
            (actor_user_id, event_type, resource_type, resource_id, outcome, details)
         VALUES ($1, 'storage.volume.planned', 'storage_volume', $2, 'success',
                 jsonb_build_object('size_gib', $3, 'format', $4))",
    )
    .bind(user.id)
    .bind(volume.id.to_string())
    .bind(volume.size_gib)
    .bind(&volume.format)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json(volume)))
}

pub async fn delete_volume(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let mut transaction = pool.begin().await?;
    let _quota = ensure_quota_row(&mut transaction, user.id).await?;
    let volume = sqlx::query_as::<_, StorageVolume>(
        "SELECT id, reservation_id, name, size_gib, format, status, created_at, updated_at
         FROM storage_volumes WHERE id = $1 AND user_id = $2 FOR UPDATE",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ApiError::NotFound("storage volume was not found".to_owned()))?;

    match volume.status.as_str() {
        "deleted" => {}
        "planned" | "failed" => {
            sqlx::query(
                "UPDATE storage_volumes SET status = 'deleted', updated_at = NOW() WHERE id = $1",
            )
            .bind(id)
            .execute(&mut *transaction)
            .await?;
            sqlx::query(
                "INSERT INTO audit_events
                    (actor_user_id, event_type, resource_type, resource_id, outcome)
                 VALUES ($1, 'storage.volume.plan_deleted', 'storage_volume', $2, 'success')",
            )
            .bind(user.id)
            .bind(id.to_string())
            .execute(&mut *transaction)
            .await?;
        }
        _ => {
            return Err(ApiError::Conflict(
                "A volume managed by a storage provider cannot be deleted through the metadata-only API".to_owned(),
            ));
        }
    }

    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::{validate_idempotency_key, validate_volume_name};

    #[test]
    fn accepts_safe_idempotency_keys() {
        assert_eq!(
            validate_idempotency_key("req-2026_11.17:abc").unwrap(),
            "req-2026_11.17:abc"
        );
    }

    #[test]
    fn rejects_missing_or_unsafe_idempotency_key_values() {
        assert!(validate_idempotency_key("short").is_err());
        assert!(validate_idempotency_key("../private-key").is_err());
        assert!(validate_idempotency_key(&"a".repeat(129)).is_err());
    }

    #[test]
    fn accepts_safe_volume_names() {
        assert_eq!(
            validate_volume_name("root-disk_01").unwrap(),
            "root-disk_01"
        );
    }

    #[test]
    fn rejects_unsafe_volume_names() {
        for name in ["../escape", "UpperCase", "disk/name", "disk name", ""] {
            assert!(validate_volume_name(name).is_err());
        }
    }
}
