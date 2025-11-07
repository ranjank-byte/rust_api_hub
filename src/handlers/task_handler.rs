//! HTTP handlers for Task endpoints.
//!
//! This file includes handlers and small helpers used by integration tests.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde_json::json;
use uuid::Uuid;

use crate::models::repository::TaskRepository;
use crate::models::task::{Task, TaskCreate, TaskUpdate};
use crate::utils::logger::log_info;
use serde::Deserialize;

type AppState = TaskRepository;

/// Create a task: POST /tasks
pub async fn create_task(
    State(repo): State<AppState>,
    Json(payload): Json<TaskCreate>,
) -> (StatusCode, Json<Task>) {
    log_info("create_task called");
    let task = Task::new_full(&payload.title, &payload.description);
    repo.insert(task.clone());
    (StatusCode::CREATED, Json(task))
}

/// Query params for GET /tasks
#[derive(Debug, Deserialize)]
pub struct FilterParams {
    pub completed: Option<bool>,
}

/// List tasks: GET /tasks
pub async fn get_tasks(
    State(repo): State<AppState>,
    Query(params): Query<FilterParams>,
) -> Json<Vec<Task>> {
    log_info(&format!("get_tasks called filter={:?}", params));
    let mut all = repo.list();
    // apply completed filter if present
    if let Some(completed_val) = params.completed {
        all.retain(|t| t.completed == completed_val);
    }
    // sort by title to keep response stable (easier testing)
    all.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Json(all)
}

/// Get a task by id: GET /tasks/{id}
pub async fn get_task(
    Path(id): Path<String>,
    State(repo): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info(&format!("get_task called id={}", id));
    match Uuid::parse_str(&id) {
        Ok(uuid) => match repo.get(&uuid) {
            Some(t) => (StatusCode::OK, Json(json!({"task": t}))),
            None => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))),
        },
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid uuid"})),
        ),
    }
}

/// Update a task: PUT /tasks/{id}
pub async fn update_task(
    Path(id): Path<String>,
    State(repo): State<AppState>,
    Json(payload): Json<TaskUpdate>,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info(&format!("update_task called id={}", id));
    match Uuid::parse_str(&id) {
        Ok(uuid) => match repo.update(&uuid, payload.clone()) {
            Some(t) => (StatusCode::OK, Json(json!({"task": t}))),
            None => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))),
        },
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid uuid"})),
        ),
    }
}

/// Delete a task: DELETE /tasks/{id}
pub async fn delete_task(
    Path(id): Path<String>,
    State(repo): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info(&format!("delete_task called id={}", id));
    match Uuid::parse_str(&id) {
        Ok(uuid) => {
            if repo.remove(&uuid) {
                (StatusCode::NO_CONTENT, Json(json!({})))
            } else {
                (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
            }
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid uuid"})),
        ),
    }
}

/// Count tasks: GET /tasks/count
pub async fn count_tasks(State(repo): State<AppState>) -> Json<serde_json::Value> {
    log_info("count_tasks called");
    let n = repo.count();
    Json(json!({"count": n}))
}

/// Bulk delete tasks: DELETE /tasks
/// Accepts a JSON array of UUID strings and removes any matching tasks.
/// Returns JSON {"deleted": N} where N is the number of tasks deleted.
pub async fn bulk_delete_tasks(
    State(repo): State<AppState>,
    Json(payload): Json<Vec<String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info("bulk_delete_tasks called");

    // parse valid UUIDs, ignore invalid entries
    let mut ids = Vec::with_capacity(payload.len());
    for s in payload.iter() {
        if let Ok(u) = Uuid::parse_str(s) {
            ids.push(u);
        }
    }

    let removed = if ids.is_empty() {
        0
    } else {
        repo.remove_many(&ids)
    };

    (StatusCode::OK, Json(json!({"deleted": removed})))
}

// unit tests moved to `tests/handler_tests.rs` as integration tests
