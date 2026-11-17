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
         WHERE user_id = $1 AND status IN ('provisioning', 'ready', 'deleting')",
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
         WHERE user_id = $1 AND status IN ('provisioning', 'ready', 'deleting')",
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

#[cfg(test)]
mod tests {
    use super::validate_idempotency_key;

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
}
