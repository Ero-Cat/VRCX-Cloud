//! Static SPA serving with HashRouter-friendly fallback.
//!
//! Serves the built frontend from the configured dist directory. Because
//! the app routes via the URL hash, every non-API path can safely fall
//! back to `index.html`.

use std::path::PathBuf;

use axum::extract::State;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};

#[derive(Clone)]
pub struct StaticFiles {
    dist_dir: PathBuf,
}

impl StaticFiles {
    pub fn new(dist_dir: PathBuf) -> Self {
        Self { dist_dir }
    }

    pub async fn serve(&self, uri: Uri) -> Response {
        let path = uri.path().trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };
        if path.contains("..") {
            return (StatusCode::BAD_REQUEST, "invalid path").into_response();
        }
        let file_path = self.dist_dir.join(path);
        match tokio::fs::read(&file_path).await {
            Ok(bytes) => {
                let mime = mime_for(path);
                (
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, mime),
                        (header::CACHE_CONTROL, cache_control_for(path)),
                    ],
                    bytes,
                )
                    .into_response()
            }
            Err(_) => match tokio::fs::read(self.dist_dir.join("index.html")).await {
                Ok(bytes) => (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    bytes,
                )
                    .into_response(),
                Err(_) => (
                    StatusCode::NOT_FOUND,
                    "frontend dist not found; set VRCX_CLOUD_DIST_DIR",
                )
                    .into_response(),
            },
        }
    }
}

pub async fn static_endpoint(State(files): State<StaticFiles>, uri: Uri) -> Response {
    files.serve(uri).await
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("woff2") => "font/woff2",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn cache_control_for(path: &str) -> &'static str {
    if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    }
}
