use rust_api_hub::models::task::{Task, TaskUpdate};

#[test]
fn test_create_task_properties() {
    let t = Task::new_full("title1", "desc1");
    assert_eq!(t.title, "title1");
    assert_eq!(t.description, "desc1");
    assert_eq!(t.completed, false);
}

#[test]
fn test_apply_title_update() {
    let mut t = Task::new_full("a", "b");
    let upd = TaskUpdate {
        title: Some("AA".to_string()),
        description: None,
        completed: None,
    };
    let new = t.apply_update(upd);
    assert_eq!(new.title, "AA");
    assert_eq!(new.description, "b");
}

#[test]
fn test_apply_all_update() {
    let mut t = Task::new_full("x", "y");
    let upd = TaskUpdate {
        title: Some("X".to_string()),
        description: Some("Y".to_string()),
        completed: Some(true),
    };
    let new = t.apply_update(upd);
    assert_eq!(new.title, "X");
    assert_eq!(new.description, "Y");
    assert!(new.completed);
}

#[test]
fn test_task_equality_clone() {
    let t = Task::new_full("t", "d");
    let a = t.clone();
    assert_eq!(a, t);
}

#[test]
fn test_uuid_uniqueness() {
    let a = Task::new_full("1", "1");
    let b = Task::new_full("2", "2");
    assert_ne!(a.id, b.id);
}
