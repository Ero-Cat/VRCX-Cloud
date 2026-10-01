//! Cached image serving: `GET /api/img/{file_id}/{version}`.
//!
//! Replaces the desktop `vrcx-0-img` custom protocol for world/avatar
//! thumbnails and user icons resolved from the server's image cache
//! (`ImageCache/{file_id}/{version}.png`).

use std::path::{Path, PathBuf};

use axum::extract::{Path as AxumPath, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

use super::auth::AuthContext;

pub async fn img_endpoint(
    State(ctx): State<Arc<AuthContext>>,
    AxumPath((file_id, version)): AxumPath<(String, String)>,
) -> Response {
    let cache_dir = ctx.state.app_data_path().join("ImageCache");
    // Reject traversal: components must be plain cache segments.
    if !is_safe_cache_component(&file_id) || !is_safe_cache_component(&version) {
        return (StatusCode::BAD_REQUEST, "invalid image key").into_response();
    }
    let path = cache_dir.join(&file_id).join(format!("{version}.png"));
    match tokio::fs::read(&path).await {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "private, max-age=86400"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "image not cached").into_response(),
    }
}

/// Mirror the media crate's cache component rules: alphanumerics and a
/// few safe separators only, no leading dots or path separators.
fn is_safe_cache_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && !value.starts_with('.')
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && !value.contains("..")
}

#[allow(dead_code)]
fn assert_paths_used(_p: &Path) {
    let _base: PathBuf = PathBuf::new();
}
