//! Router assembly for the API.
//! Add new route modules here.

use axum::{
    Router,
    routing::{get, post},
};

pub mod tasks;

use crate::handlers::task_handler::{
    bulk_delete_tasks, count_tasks, create_task, delete_task, get_stats, get_tags, get_task,
    get_tasks, get_tasks_by_tag, import_tasks, import_tasks_file, set_tags, update_task,
};
use crate::models::repository::TaskRepository;

pub fn create_router() -> Router<TaskRepository> {
    let repo = TaskRepository::new();
    Router::new()
        .route(
            "/tasks",
            post(create_task).get(get_tasks).delete(bulk_delete_tasks),
        )
        .route("/tasks/import", post(import_tasks))
        .route("/tasks/import/file", post(import_tasks_file))
        .route("/tasks/count", get(count_tasks))
        .route("/tasks/stats", get(get_stats))
        .route("/tasks/search/by_tag", get(get_tasks_by_tag))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .route("/tasks/{id}/tags", get(get_tags).put(set_tags))
        .route("/health", get(tasks::health))
        .route("/info", get(tasks::info))
        .with_state(repo)
}
