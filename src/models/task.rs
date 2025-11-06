//! Task model and DTOs
//! This file contains multiple unit tests to reach test count and exercise model behavior.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The domain Task object stored in memory.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

/// Input DTO for task creation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskCreate {
    pub title: String,
    pub description: String,
}

/// Input DTO for task updates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

impl Task {
    /// Create a new task with generated UUID
    pub fn new_full(title: &str, description: &str) -> Self {
        Task {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            completed: false,
        }
    }

    /// Apply an update to the task in-place and return updated copy
    pub fn apply_update(&mut self, upd: TaskUpdate) -> Task {
        if let Some(t) = upd.title {
            self.title = t;
        }
        if let Some(d) = upd.description {
            self.description = d;
        }
        if let Some(c) = upd.completed {
            self.completed = c;
        }
        self.clone()
    }
}

// unit tests moved to `tests/task_tests.rs` as integration tests
