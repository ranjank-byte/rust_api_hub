//! HTTP handlers for Task endpoints.
//!
//! This file includes handlers and small helpers used by integration tests.

use axum::body::Bytes;
use axum::http::HeaderMap;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use csv::ReaderBuilder;
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
pub struct ListParams {
    pub completed: Option<bool>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub sort: Option<String>,
}

/// List tasks: GET /tasks
/// Supports optional filters: completed, pagination (page, per_page), and sorting (sort=created_at[:asc|:desc]).
pub async fn get_tasks(
    State(repo): State<AppState>,
    Query(params): Query<ListParams>,
) -> Json<serde_json::Value> {
    log_info(&format!("get_tasks called params={:?}", params));

    // defaults and validation
    let page = params.page.unwrap_or(1).max(1);
    let per_page_requested = params.per_page.unwrap_or(20).max(1);
    let per_page_cap = 100usize;
    let per_page = per_page_requested.min(per_page_cap);

    // determine sort order
    let mut desc = false;
    if let Some(s) = params.sort.as_deref() {
        // accept "created_at" or "created_at:asc"/":desc"
        if s.starts_with("created_at") && s.ends_with(":desc") {
            desc = true;
        }
    }

    // get sorted list by created_at (server-side sort)
    let mut items = repo.list_sorted_by_created_at(desc);

    // apply completed filter if present
    if let Some(completed_val) = params.completed {
        items.retain(|t| t.completed == completed_val);
    }

    let total = items.len();
    // pagination: page is 1-based
    let start = per_page * (page.saturating_sub(1));
    let end = usize::min(start + per_page, total);
    let page_items = if start >= total {
        Vec::new()
    } else {
        items[start..end].to_vec()
    };

    Json(json!({
        "items": page_items,
        "total": total,
        "page": page,
        "per_page": per_page
    }))
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

/// Import tasks from a JSON array POST /tasks/import (application/json)
pub async fn import_tasks_json(
    State(repo): State<AppState>,
    Json(payload): Json<Vec<TaskCreate>>,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info(&format!(
        "import_tasks_json called payload_len={}",
        payload.len()
    ));
    let created = repo.insert_many(&payload);
    (
        StatusCode::CREATED,
        Json(json!({"imported": created.len(), "tasks": created})),
    )
}

/// Import tasks from CSV POST /tasks/import/csv (text/csv)
/// Expects header row with `title,description` and optional additional columns ignored by the CSV deserializer.
pub async fn import_tasks_csv(
    State(repo): State<AppState>,
    body: Bytes,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info("import_tasks_csv called");
    let s = match std::str::from_utf8(&body) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid utf8 in body"})),
            );
        }
    };

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(s.as_bytes());

    let mut creates: Vec<TaskCreate> = Vec::new();
    for result in reader.deserialize::<TaskCreate>() {
        match result {
            Ok(tc) => creates.push(tc),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": format!("csv parse error: {}", e)})),
                );
            }
        }
    }

    let created = repo.insert_many(&creates);
    (
        StatusCode::CREATED,
        Json(json!({"imported": created.len(), "tasks": created})),
    )
}

/// Unified import: POST /tasks/import
/// Accepts either `application/json` (array of TaskCreate) or `text/csv` (with header).
/// Returns a partial-success summary: { imported, failed, errors, tasks } with 201.
pub async fn import_tasks(
    State(repo): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info("import_tasks called");

    let ct = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let mut valid: Vec<TaskCreate> = Vec::new();
    let mut errors: Vec<serde_json::Value> = Vec::new();

    if ct.contains("json") || ct.is_empty() {
        // try JSON
        match serde_json::from_slice::<Vec<TaskCreate>>(&body) {
            Ok(items) => {
                for (i, it) in items.into_iter().enumerate() {
                    match it.validate() {
                        Ok(_) => valid.push(it),
                        Err(e) => errors.push(json!({"index": i, "error": e})),
                    }
                }
            }
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": format!("json parse error: {}", e)})),
                );
            }
        }
    } else if ct.contains("csv") {
        // CSV path
        let s = match std::str::from_utf8(&body) {
            Ok(v) => v,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "invalid utf8 in body"})),
                );
            }
        };

        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(s.as_bytes());
        for (i, dec) in reader.deserialize::<TaskCreate>().enumerate() {
            match dec {
                Ok(tc) => match tc.validate() {
                    Ok(_) => valid.push(tc),
                    Err(e) => errors.push(json!({"row": i + 1, "error": e})),
                },
                Err(e) => {
                    errors.push(json!({"row": i + 1, "error": format!("csv parse error: {}", e)}))
                }
            }
        }
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "unsupported content-type"})),
        );
    }

    // persist valid rows
    let created = if valid.is_empty() {
        Vec::new()
    } else {
        repo.insert_many(&valid)
    };

    let imported = created.len();
    let failed = errors.len();

    (
        StatusCode::CREATED,
        Json(json!({
            "imported": imported,
            "failed": failed,
            "errors": errors,
            "tasks": created
        })),
    )
}

// unit tests moved to `tests/handler_tests.rs` as integration tests
