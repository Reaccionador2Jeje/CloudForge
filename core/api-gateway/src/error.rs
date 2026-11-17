use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("The request is invalid: {0}")]
    Validation(String),
    #[error("Database is not configured")]
    DatabaseNotConfigured,
    #[error("A database operation failed")]
    Database(#[from] sqlx::Error),
    #[error("Authentication is required")]
    Unauthorized,
    #[error("The requested resource was not found: {0}")]
    NotFound(String),
    #[error("The request conflicts with current resource state: {0}")]
    Conflict(String),
    #[error("The account lacks the required permission")]
    Forbidden,
    #[error("Administrator bootstrap is not configured")]
    BootstrapDisabled,
    #[error("Initial administrator setup has already been completed")]
    BootstrapAlreadyCompleted,
    #[error("An internal service error occurred")]
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Validation(message) => (StatusCode::BAD_REQUEST, "validation_error", message),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "A valid bearer session is required".to_owned(),
            ),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            Self::Conflict(message) => (StatusCode::CONFLICT, "conflict", message),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "The account lacks the required permission".to_owned(),
            ),
            Self::BootstrapDisabled => (
                StatusCode::SERVICE_UNAVAILABLE,
                "bootstrap_disabled",
                "Administrator bootstrap is not configured".to_owned(),
            ),
            Self::BootstrapAlreadyCompleted => (
                StatusCode::CONFLICT,
                "bootstrap_completed",
                "The initial administrator has already been created".to_owned(),
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "An internal service error occurred".to_owned(),
            ),
            Self::DatabaseNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "database_not_configured",
                "This operation requires a configured database".to_owned(),
            ),
            Self::Database(error) => {
                tracing::error!(%error, "Database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal service error occurred".to_owned(),
                )
            }
        };

        (
            status,
            Json(ErrorBody {
                error: code,
                message,
            }),
        )
            .into_response()
    }
}
