use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("The requested resource was not found")]
    NotFound,
    #[error("The request is invalid: {0}")]
    Validation(String),
    #[error("Database is not configured")]
    DatabaseNotConfigured,
    #[error("A database operation failed")]
    Database(#[from] sqlx::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "The requested resource was not found".to_owned(),
            ),
            Self::Validation(message) => (StatusCode::BAD_REQUEST, "validation_error", message),
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
