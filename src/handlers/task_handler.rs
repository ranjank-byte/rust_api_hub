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
    // tags not provided via creation DTO (legacy tests). Accept optional header 'x-tags'
    // with comma-separated list of tags for future clients.
    // NOTE: This is a placeholder; will be expanded when DTO evolves.
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

    // filter by tag if provided
    // Tag filter available via dedicated endpoint: GET /tasks/search/by_tag

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

/// Import tasks by uploading a multipart/form-data file (field name `file`).
/// This is a simple, non-streaming parser: the entire request body is read into memory.
/// It enforces a size limit to avoid OOM for very large uploads.
pub async fn import_tasks_file(
    State(repo): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info("import_tasks_file called");

    const MAX_BYTES: usize = 5 * 1024 * 1024; // 5 MB
    if body.len() > MAX_BYTES {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({"error": "payload too large"})),
        );
    }

    // extract boundary from content-type
    let ct = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let boundary = if let Some(idx) = ct.find("boundary=") {
        &ct[idx + "boundary=".len()..]
    } else {
        ""
    };

    if !ct.contains("multipart/form-data") || boundary.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "expected multipart/form-data with boundary"})),
        );
    }

    // crude split by boundary; each part begins with `--{boundary}`
    let raw = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid utf8 in body"})),
            );
        }
    };

    let marker = format!("--{}", boundary.trim());
    let parts: Vec<&str> = raw.split(&marker).collect();
    let mut file_content_opt: Option<&str> = None;

    for part in parts.iter() {
        // skip preamble and epilogue
        if part.trim().is_empty() || part.trim() == "--" {
            continue;
        }

        // find Content-Disposition header with name="file"
        if part.contains("name=\"file\"") {
            // part looks like: \r\nContent-Disposition: form-data; name="file"; filename="..."\r\nContent-Type: text/csv\r\n\r\n<file-body>\r\n
            if let Some(idx) = part.find("\r\n\r\n") {
                let body_start = idx + 4;
                let body_end = part.len();
                let file_body = &part[body_start..body_end];
                // strip trailing CRLF and possible ending --
                let file_body = file_body.trim_end_matches('\r').trim_end_matches('\n');
                file_content_opt = Some(file_body);
                break;
            }
        }
    }

    let file_content = match file_content_opt {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "file part not found"})),
            );
        }
    };

    // parse CSV from file_content
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file_content.as_bytes());
    let mut valid: Vec<TaskCreate> = Vec::new();
    let mut errors: Vec<serde_json::Value> = Vec::new();
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

    let created = if valid.is_empty() {
        Vec::new()
    } else {
        repo.insert_many(&valid)
    };
    let imported = created.len();
    let failed = errors.len();

    (
        StatusCode::CREATED,
        Json(json!({"imported": imported, "failed": failed, "errors": errors, "tasks": created})),
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

// ------------------------
// Tags management endpoints
// ------------------------

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct TagsPayload {
    pub tags: Vec<String>,
}

fn validate_tags(tags: &[String]) -> Result<(), String> {
    for t in tags.iter() {
        if t.trim().is_empty() {
            return Err("tags must not contain empty entries".into());
        }
        if t.len() > 64 {
            return Err("tag too long (max 64 chars)".into());
        }
    }
    Ok(())
}

fn normalize_tags(tags: &[String]) -> Vec<String> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for t in tags.iter() {
        let norm = t.trim().to_lowercase();
        if !norm.is_empty() && seen.insert(norm.clone()) {
            out.push(norm);
        }
    }
    out
}

/// Replace tags on a task: PUT /tasks/{id}/tags
pub async fn set_tags(
    Path(id): Path<String>,
    State(repo): State<AppState>,
    Json(payload): Json<TagsPayload>,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info(&format!("set_tags called id={}", id));

    if let Err(e) = validate_tags(&payload.tags) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": e})));
    }

    let tags = normalize_tags(&payload.tags);

    match Uuid::parse_str(&id) {
        Ok(uuid) => {
            let mut t = match repo.get(&uuid) {
                Some(existing) => existing,
                None => return (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))),
            };
            t.tags = tags;
            // persist by calling update with no field changes other than tags
            let _ = repo.update(
                &uuid,
                TaskUpdate {
                    title: None,
                    description: None,
                    completed: None,
                },
            );
            // Directly overwrite stored task with updated tags to avoid changing TaskUpdate DTO
            repo.insert(t.clone());
            (StatusCode::OK, Json(json!({"task": t})))
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid uuid"})),
        ),
    }
}

/// Get tags of a task: GET /tasks/{id}/tags
pub async fn get_tags(
    Path(id): Path<String>,
    State(repo): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    log_info(&format!("get_tags called id={}", id));
    match Uuid::parse_str(&id) {
        Ok(uuid) => match repo.get(&uuid) {
            Some(t) => (StatusCode::OK, Json(json!({"tags": t.tags}))),
            None => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))),
        },
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid uuid"})),
        ),
    }
}

/// Query tasks by tag: GET /tasks/search/by_tag?tag=...
#[derive(Debug, Deserialize)]
pub struct TagQuery {
    pub tag: String,
}

pub async fn get_tasks_by_tag(
    State(repo): State<AppState>,
    Query(q): Query<TagQuery>,
) -> Json<serde_json::Value> {
    log_info(&format!("get_tasks_by_tag called tag={}", q.tag));
    let tag = q.tag.to_lowercase();
    let mut items = repo.list();
    items.retain(|t| t.tags.iter().any(|x| x.eq_ignore_ascii_case(&tag)));
    Json(json!({"items": items, "total": items.len()}))
}

// ------------------------
// Task statistics/analytics
// ------------------------

/// Statistics summary: GET /tasks/stats
/// Returns aggregated metrics about the task repository:
/// - total, completed, incomplete counts
/// - tag_distribution: top N tags with counts (sorted descending)
/// - oldest_created_at, newest_created_at (ISO timestamps)
pub async fn get_stats(State(repo): State<AppState>) -> Json<serde_json::Value> {
    log_info("get_stats called");
    let items = repo.list();
    let total = items.len();
    let completed = items.iter().filter(|t| t.completed).count();
    let incomplete = total - completed;

    // Build tag frequency map
    use std::collections::HashMap;
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for task in items.iter() {
        for tag in task.tags.iter() {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    // Sort tags by frequency (descending), then alphabetically for ties
    let mut tag_vec: Vec<(String, usize)> = tag_counts.into_iter().collect();
    tag_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    // Limit to top 10 tags
    let top_tags: Vec<serde_json::Value> = tag_vec
        .iter()
        .take(10)
        .map(|(tag, count)| json!({"tag": tag, "count": count}))
        .collect();

    // Find oldest and newest by created_at
    let oldest_opt = items.iter().min_by_key(|t| t.created_at);
    let newest_opt = items.iter().max_by_key(|t| t.created_at);

    let oldest_created = oldest_opt.map(|t| t.created_at.to_rfc3339());
    let newest_created = newest_opt.map(|t| t.created_at.to_rfc3339());

    Json(json!({
        "total": total,
        "completed": completed,
        "incomplete": incomplete,
        "tag_distribution": top_tags,
        "oldest_created_at": oldest_created,
        "newest_created_at": newest_created,
    }))
}

// unit tests moved to `tests/handler_tests.rs` as integration tests
