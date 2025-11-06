//! Additional task route helpers and example health check route.
//! Kept as a separate module to give more PR surface area later.

use axum::response::Json;
use axum::{Router, routing::get};
use serde_json::json;

pub fn routes() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/info", get(info))
}

/// Simple health check
pub(crate) async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

/// Lightweight info endpoint
pub(crate) async fn info() -> Json<serde_json::Value> {
    Json(json!({
        "name": "rust_api_hub",
        "version": "0.1.0",
        "desc": "Axum-based task API"
    }))
}

// unit tests moved to `tests/routes_tasks_tests.rs` as integration tests
