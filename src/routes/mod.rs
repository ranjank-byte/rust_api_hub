//! Router assembly for the API.
//! Add new route modules here.

use axum::{
    Router,
    routing::{get, post},
};

pub mod tasks;

use crate::handlers::task_handler::{create_task, delete_task, get_task, get_tasks, update_task};
use crate::models::repository::TaskRepository;

pub fn create_router() -> Router<TaskRepository> {
    let repo = TaskRepository::new();
    Router::new()
        .route("/tasks", post(create_task).get(get_tasks))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .route("/health", get(tasks::health))
        .route("/info", get(tasks::info))
        .with_state(repo)
}
