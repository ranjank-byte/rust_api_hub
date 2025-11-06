//! In-memory task repository.
//! Uses `parking_lot::RwLock` for simple concurrency (faster and smaller than std::sync).

use crate::models::task::{Task, TaskUpdate};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Simple thread-safe repository wrapper
#[derive(Clone)]
pub struct TaskRepository {
    inner: Arc<RwLock<HashMap<Uuid, Task>>>,
}

impl TaskRepository {
    pub fn new() -> Self {
        TaskRepository {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, task: Task) {
        let mut m = self.inner.write();
        m.insert(task.id, task);
    }

    pub fn get(&self, id: &Uuid) -> Option<Task> {
        let m = self.inner.read();
        m.get(id).cloned()
    }

    pub fn list(&self) -> Vec<Task> {
        let m = self.inner.read();
        m.values().cloned().collect()
    }

    pub fn update(&self, id: &Uuid, upd: TaskUpdate) -> Option<Task> {
        let mut m = self.inner.write();
        if let Some(t) = m.get_mut(id) {
            let updated = t.apply_update(upd);
            Some(updated)
        } else {
            None
        }
    }

    pub fn remove(&self, id: &Uuid) -> bool {
        let mut m = self.inner.write();
        m.remove(id).is_some()
    }
}

impl Default for TaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

// unit tests moved to `tests/repository_tests.rs` as integration tests
