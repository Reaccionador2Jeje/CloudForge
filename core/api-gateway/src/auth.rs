use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::{error::ApiError, instances::AppState};

const SESSION_HOURS: i64 = 12;
const BOOTSTRAP_LOCK_ID: i64 = 7_340_021_991;

#[derive(Debug, Deserialize)]
pub struct BootstrapRequest {
    pub bootstrap_token: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_at: chrono::DateTime<Utc>,
    pub user: UserView,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserView {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, sqlx::FromRow)]
struct StoredUser {
    id: Uuid,
    email: String,
    display_name: String,
    password_hash: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
}

#[derive(Clone)]
pub struct PermissionGate {
    pub state: AppState,
    pub resource: &'static str,
}

fn normalize_email(email: &str) -> Result<String, ApiError> {
    let value = email.trim().to_ascii_lowercase();
    if value.len() < 3 || value.len() > 254 || !value.contains('@') {
        return Err(ApiError::Validation(
            "A valid email address is required".to_owned(),
        ));
    }
    Ok(value)
}

fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < 12 || password.len() > 128 {
        return Err(ApiError::Validation(
            "Password must contain between 12 and 128 bytes".to_owned(),
        ));
    }
    Ok(())
}

fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn new_session_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bearer_token(request: &Request<axum::body::Body>) -> Option<&str> {
    let value = request.headers().get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") || token.is_empty() || token.contains(' ') {
        return None;
    }
    Some(token)
}

async fn record_auth_event(
    pool: &sqlx::PgPool,
    actor_id: Option<Uuid>,
    event_type: &str,
    outcome: &str,
) {
    if let Err(error) = sqlx::query(
        "INSERT INTO audit_events (actor_user_id, event_type, resource_type, outcome)
         VALUES ($1, $2, 'auth', $3)",
    )
    .bind(actor_id)
    .bind(event_type)
    .bind(outcome)
    .execute(pool)
    .await
    {
        tracing::error!(%error, "Unable to write authentication audit event");
    }
}

pub async fn bootstrap(
    State(state): State<AppState>,
    Json(request): Json<BootstrapRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let configured_token = std::env::var("CLOUDFORGE_BOOTSTRAP_TOKEN")
        .ok()
        .filter(|value| value.len() >= 32)
        .ok_or(ApiError::BootstrapDisabled)?;
    let supplied_hash = Sha256::digest(request.bootstrap_token.as_bytes());
    let configured_hash = Sha256::digest(configured_token.as_bytes());
    if supplied_hash[..].ct_eq(&configured_hash[..]).unwrap_u8() != 1 {
        record_auth_event(pool, None, "auth.bootstrap.denied", "denied").await;
        return Err(ApiError::Unauthorized);
    }

    let email = normalize_email(&request.email)?;
    validate_password(&request.password)?;
    let display_name = request.display_name.trim();
    if display_name.is_empty() || display_name.len() > 80 {
        return Err(ApiError::Validation(
            "Display name must contain between 1 and 80 bytes".to_owned(),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(|_| ApiError::Internal)?
        .to_string();

    let mut transaction = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(BOOTSTRAP_LOCK_ID)
        .execute(&mut *transaction)
        .await?;

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *transaction)
        .await?;
    if user_count != 0 {
        transaction.rollback().await?;
        record_auth_event(pool, None, "auth.bootstrap.denied", "denied").await;
        return Err(ApiError::BootstrapAlreadyCompleted);
    }

    let user = sqlx::query_as::<_, UserView>(
        "INSERT INTO users (email, password_hash, display_name, status)
         VALUES ($1, $2, $3, 'active')
         RETURNING id, email, display_name",
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(display_name)
    .fetch_one(&mut *transaction)
    .await?;

    let role_id: Uuid = sqlx::query_scalar("SELECT id FROM roles WHERE name = 'platform_admin'")
        .fetch_one(&mut *transaction)
        .await?;
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
        .bind(user.id)
        .bind(role_id)
        .execute(&mut *transaction)
        .await?;

    let access_token = new_session_token();
    let expires_at = Utc::now() + Duration::hours(SESSION_HOURS);
    sqlx::query(
        "INSERT INTO user_sessions (user_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(hash_token(&access_token))
    .bind(expires_at)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "INSERT INTO audit_events (actor_user_id, event_type, resource_type, resource_id, outcome)
         VALUES ($1, 'auth.bootstrap.completed', 'user', $2, 'success')",
    )
    .bind(user.id)
    .bind(user.id.to_string())
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(SessionResponse {
            access_token,
            token_type: "Bearer",
            expires_at,
            user,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let email = normalize_email(&request.email)?;
    let user = sqlx::query_as::<_, StoredUser>(
        "SELECT id, email, display_name, password_hash
         FROM users WHERE email = $1 AND status = 'active'",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    let Some(user) = user else {
        record_auth_event(pool, None, "auth.login", "failure").await;
        return Err(ApiError::Unauthorized);
    };
    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| ApiError::Internal)?;
    if Argon2::default()
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        record_auth_event(pool, Some(user.id), "auth.login", "failure").await;
        return Err(ApiError::Unauthorized);
    }

    let access_token = new_session_token();
    let expires_at = Utc::now() + Duration::hours(SESSION_HOURS);
    sqlx::query(
        "INSERT INTO user_sessions (user_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(hash_token(&access_token))
    .bind(expires_at)
    .execute(pool)
    .await?;
    record_auth_event(pool, Some(user.id), "auth.login", "success").await;

    Ok(Json(SessionResponse {
        access_token,
        token_type: "Bearer",
        expires_at,
        user: UserView {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        },
    }))
}

async fn authenticate(state: &AppState, token: &str) -> Result<AuthenticatedUser, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    sqlx::query_as::<_, AuthenticatedUser>(
        "SELECT u.id, u.email, u.display_name
         FROM user_sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > NOW()
           AND u.status = 'active'",
    )
    .bind(hash_token(token))
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::Unauthorized)
}

pub async fn me(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
) -> Result<Json<UserView>, ApiError> {
    let token = bearer_token(&request).ok_or(ApiError::Unauthorized)?;
    let user = authenticate(&state, token).await?;
    Ok(Json(UserView {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
) -> Result<StatusCode, ApiError> {
    let token = bearer_token(&request).ok_or(ApiError::Unauthorized)?;
    let user = authenticate(&state, token).await?;
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    sqlx::query(
        "UPDATE user_sessions SET revoked_at = NOW()
         WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(hash_token(token))
    .execute(pool)
    .await?;
    record_auth_event(pool, Some(user.id), "auth.logout", "success").await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn require_session(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let Some(token) = bearer_token(&request).map(str::to_owned) else {
        return ApiError::Unauthorized.into_response();
    };
    match authenticate(&state, &token).await {
        Ok(user) => {
            let mut request = request;
            request.extensions_mut().insert(user);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}

pub async fn require_permission(
    State(gate): State<PermissionGate>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let Some(token) = bearer_token(&request).map(str::to_owned) else {
        return ApiError::Unauthorized.into_response();
    };
    let user = match authenticate(&gate.state, &token).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    let Some(pool) = gate.state.pool.as_ref() else {
        return ApiError::DatabaseNotConfigured.into_response();
    };
    let action = if request.method() == axum::http::Method::POST {
        "create"
    } else {
        "read"
    };
    let permitted: Result<bool, sqlx::Error> = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM user_roles ur
            JOIN role_permissions rp ON rp.role_id = ur.role_id
            JOIN permissions p ON p.id = rp.permission_id
            WHERE ur.user_id = $1 AND p.resource = $2 AND p.action = $3
         )",
    )
    .bind(user.id)
    .bind(gate.resource)
    .bind(action)
    .fetch_one(pool)
    .await;
    match permitted {
        Ok(true) => {
            let mut request = request;
            request.extensions_mut().insert(user);
            next.run(request).await
        }
        Ok(false) => {
            record_auth_event(pool, Some(user.id), "auth.permission.denied", "denied").await;
            ApiError::Forbidden.into_response()
        }
        Err(error) => {
            tracing::error!(%error, "Permission lookup failed");
            ApiError::Internal.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{hash_token, normalize_email, validate_password};

    #[test]
    fn session_token_hash_is_stable_and_not_the_token() {
        let token = "a-long-random-session-token";
        assert_eq!(hash_token(token), hash_token(token));
        assert_ne!(hash_token(token), token);
        assert_eq!(hash_token(token).len(), 64);
    }

    #[test]
    fn email_is_normalized() {
        assert_eq!(
            normalize_email("  ADMIN@EXAMPLE.COM ").unwrap(),
            "admin@example.com"
        );
    }

    #[test]
    fn rejects_short_passwords() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("long-enough-password").is_ok());
    }
}
