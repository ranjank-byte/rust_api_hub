use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use rust_api_hub::handlers::task_handler::{bulk_delete_tasks, create_task};
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::TaskCreate;

fn app_state() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn bulk_delete_none_returns_zero() {
    let repo = app_state();
    let (code, body) = bulk_delete_tasks(State(repo.clone()), Json(Vec::<String>::new())).await;
    assert_eq!(code, StatusCode::OK);
    let v = body.0;
    assert_eq!(v["deleted"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn bulk_delete_some_removes_only_specified() {
    let repo = app_state();
    // create 3 tasks
    let mut ids = Vec::new();
    for i in 0..3 {
        let payload = TaskCreate {
            title: format!("t{}", i),
            description: "d".into(),
        };
        let (code, created) = create_task(State(repo.clone()), Json(payload)).await;
        assert_eq!(code, StatusCode::CREATED);
        ids.push(created.id.to_string());
    }

    // delete first two
    let delete_ids = vec![ids[0].clone(), ids[1].clone()];
    let (code, body) = bulk_delete_tasks(State(repo.clone()), Json(delete_ids)).await;
    assert_eq!(code, StatusCode::OK);
    let v = body.0;
    assert_eq!(v["deleted"].as_u64().unwrap(), 2);

    // remaining should be 1
    let remaining = repo.list();
    assert_eq!(remaining.len(), 1);
}

#[tokio::test]
async fn bulk_delete_all_removes_everything() {
    let repo = app_state();
    let mut ids = Vec::new();
    for i in 0..5 {
        let payload = TaskCreate {
            title: format!("t{}", i),
            description: "d".into(),
        };
        let (code, created) = create_task(State(repo.clone()), Json(payload)).await;
        assert_eq!(code, StatusCode::CREATED);
        ids.push(created.id.to_string());
    }
    let (code, body) = bulk_delete_tasks(State(repo.clone()), Json(ids)).await;
    assert_eq!(code, StatusCode::OK);
    let v = body.0;
    assert_eq!(v["deleted"].as_u64().unwrap(), 5);
    assert_eq!(repo.list().len(), 0);
}
