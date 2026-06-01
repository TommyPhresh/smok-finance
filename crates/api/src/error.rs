//! Convert domain errors to HTTP responses

use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use smok_core::error::CoreError;
use serde_json::json;

pub struct ApiError(CoreError);

impl From<CoreError> for ApiError {
    fn from(e: CoreError) -> Self { ApiError(e) }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            CoreError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            CoreError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            CoreError::Database(e) => {
                tracing::error!("DB error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
