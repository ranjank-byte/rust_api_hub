use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::Task;
use uuid::Uuid;

#[test]
fn repo_insert_and_get() {
    let repo = TaskRepository::new();
    let t = Task::new_full("a", "b");
    let id = t.id;
    repo.insert(t.clone());
    let got = repo.get(&id).expect("should exist");
    assert_eq!(got.title, "a");
}

#[test]
fn repo_list_and_remove() {
    let repo = TaskRepository::new();
    let t1 = Task::new_full("1", "1");
    let t2 = Task::new_full("2", "2");
    repo.insert(t1.clone());
    repo.insert(t2.clone());
    let l = repo.list();
    assert_eq!(l.len(), 2);
    assert!(repo.remove(&t1.id));
    let l2 = repo.list();
    assert_eq!(l2.len(), 1);
}

#[test]
fn repo_update_works() {
    let repo = TaskRepository::new();
    let t = Task::new_full("x", "y");
    let id = t.id;
    repo.insert(t.clone());
    let upd = rust_api_hub::models::task::TaskUpdate {
        title: Some("Z".to_string()),
        description: None,
        completed: Some(true),
    };
    let res = repo.update(&id, upd);
    assert!(res.is_some());
    let got = repo.get(&id).unwrap();
    assert_eq!(got.title, "Z");
    assert!(got.completed);
}

#[test]
fn repo_nonexistent_update_none() {
    let repo = TaskRepository::new();
    let res = repo.update(
        &Uuid::new_v4(),
        rust_api_hub::models::task::TaskUpdate {
            title: None,
            description: None,
            completed: None,
        },
    );
    assert!(res.is_none());
}
