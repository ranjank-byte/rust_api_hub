use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use rust_api_hub::handlers::task_handler::{count_tasks, create_task};
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::TaskCreate;

fn app_state() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn count_empty_repo_is_zero() {
    let repo = app_state();
    let body = count_tasks(State(repo)).await;
    // body is Json<Value> -> {"count": 0}
    let v = body.0;
    assert_eq!(v["count"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn count_after_one_insert_is_one() {
    let repo = app_state();
    let payload = TaskCreate {
        title: "t1".into(),
        description: "d1".into(),
    };
    let (code, _created) = create_task(State(repo.clone()), Json(payload)).await;
    assert_eq!(code, StatusCode::CREATED);
    let body = count_tasks(State(repo)).await;
    let v = body.0;
    assert_eq!(v["count"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn count_after_multiple_inserts_is_n() {
    let repo = app_state();
    for i in 0..5 {
        let payload = TaskCreate {
            title: format!("t{}", i),
            description: "d".into(),
        };
        let (code, _created) = create_task(State(repo.clone()), Json(payload)).await;
        assert_eq!(code, StatusCode::CREATED);
    }
    let body = count_tasks(State(repo)).await;
    let v = body.0;
    assert_eq!(v["count"].as_u64().unwrap(), 5);
}
