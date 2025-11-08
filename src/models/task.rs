//! Task model and DTOs
//! This file contains multiple unit tests to reach test count and exercise model behavior.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// The domain Task object stored in memory.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    /// Timestamp of the most recent update to this task.
    pub updated_at: DateTime<Utc>,
}

/// Input DTO for task creation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskCreate {
    pub title: String,
    pub description: String,
}

impl TaskCreate {
    /// Basic validation for creation DTOs.
    /// Returns Err with a short message if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("title must not be empty".into());
        }
        Ok(())
    }
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
        let now = Utc::now();
        Task {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            completed: false,
            created_at: now,
            updated_at: now,
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
        // record the time of this update
        self.updated_at = Utc::now();
        self.clone()
    }

    /// Return a small JSON representation of the task including ISO timestamps.
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id.to_string(),
            "title": self.title,
            "description": self.description,
            "completed": self.completed,
            "created_at": self.created_at.to_rfc3339(),
            "updated_at": self.updated_at.to_rfc3339(),
        })
    }
}

// unit tests moved to `tests/task_tests.rs` as integration tests
