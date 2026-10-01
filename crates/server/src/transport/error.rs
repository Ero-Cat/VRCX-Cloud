//! API error model for the web transport.
//!
//! Mirrors the shape the desktop bindings taught the frontend to expect:
//! a structured `{ code, message }` body, with `unsupportedOnWeb` marking
//! desktop-only commands so the UI can degrade via host capabilities.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Desktop-only command (window/tray/clipboard/...) — the web
    /// frontend hides or stubs these via host capabilities.
    #[error("{0} is not available on the web server")]
    Unsupported(String),
    #[error("{0}")]
    Message(String),
    #[error("authentication required")]
    Unauthorized,
    #[error("invalid request: {0}")]
    BadRequest(String),
}

impl ApiError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unsupported(_) => "unsupportedOnWeb",
            Self::Message(_) => "commandFailed",
            Self::Unauthorized => "unauthorized",
            Self::BadRequest(_) => "badRequest",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Unsupported(_) | Self::Message(_) => StatusCode::OK,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<vrcx_0_composition::Error> for ApiError {
    fn from(error: vrcx_0_composition::Error) -> Self {
        Self::Message(error.to_string())
    }
}

impl From<vrcx_0_application_core::Error> for ApiError {
    fn from(error: vrcx_0_application_core::Error) -> Self {
        Self::Message(error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = Json(json!({
            "ok": false,
            "code": self.code(),
            "message": self.to_string(),
        }));
        (status, body).into_response()
    }
}
