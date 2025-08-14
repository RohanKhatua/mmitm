use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Google API error: {0}")]
    GoogleApiError(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::GoogleApiError(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::ExternalApi(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Serialization(err) => {
                tracing::error!("Serialization error: {:?}", err);
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid request format".to_string(),
                )
            }
            AppError::HttpClient(err) => {
                tracing::error!("HTTP client error: {:?}", err);
                (
                    StatusCode::BAD_GATEWAY,
                    "External service error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
