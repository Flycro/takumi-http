use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Processing error: {0}")]
    ProcessingError(#[from] takumi::Error),

    #[error("Invalid JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Multipart error: {0}")]
    MultipartError(String),

    #[error("Image decode error: {0}")]
    ImageDecodeError(String),

    #[error("Cache is disabled")]
    CacheDisabled,

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BadRequest"),
            ApiError::Validation(_) => (StatusCode::BAD_REQUEST, "ValidationError"),
            ApiError::JsonError(_) => (StatusCode::BAD_REQUEST, "JsonError"),
            ApiError::MultipartError(_) => (StatusCode::BAD_REQUEST, "MultipartError"),
            ApiError::ImageDecodeError(_) => (StatusCode::UNPROCESSABLE_ENTITY, "ImageDecodeError"),
            ApiError::CacheDisabled => (StatusCode::SERVICE_UNAVAILABLE, "CacheDisabled"),
            ApiError::ProcessingError(e) => match e {
                takumi::Error::InvalidViewport => {
                    (StatusCode::UNPROCESSABLE_ENTITY, "InvalidViewport")
                }
                takumi::Error::ImageResolveError(_) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, "ImageResolveError")
                }
                takumi::Error::FontError(_) => (StatusCode::UNPROCESSABLE_ENTITY, "FontError"),
                takumi::Error::LayoutError(_) => (StatusCode::UNPROCESSABLE_ENTITY, "LayoutError"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "ProcessingError"),
            },
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "InternalError"),
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
