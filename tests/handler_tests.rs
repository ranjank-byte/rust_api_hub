use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use rust_api_hub::handlers::task_handler::{create_task, delete_task, get_task, update_task};
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::{TaskCreate, TaskUpdate};

fn app_state() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn create_and_get_task_flow() {
    let repo = app_state();
    let payload = TaskCreate {
        title: "t1".into(),
        description: "d1".into(),
    };
    let (code, _created) = create_task(State(repo.clone()), Json(payload)).await;
    assert_eq!(code, StatusCode::CREATED);
    let items = repo.list();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "t1");
    // fetch by id
    let id = items[0].id.to_string();
    let (code2, _body) = get_task(Path(id.clone()), State(repo.clone())).await;
    assert_eq!(code2, StatusCode::OK);
    // bad id
    let (code3, _) = get_task(Path("not-a-uuid".to_string()), State(repo.clone())).await;
    assert_eq!(code3, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_nonexistent_returns_not_found() {
    let repo = app_state();
    let fake = uuid::Uuid::new_v4().to_string();
    let payload = TaskUpdate {
        title: None,
        description: None,
        completed: Some(true),
    };
    let (code, _) = update_task(Path(fake), State(repo), Json(payload)).await;
    assert_eq!(code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent_returns_not_found() {
    let repo = app_state();
    let fake = uuid::Uuid::new_v4().to_string();
    let (code, _) = delete_task(Path(fake), State(repo)).await;
    assert_eq!(code, StatusCode::NOT_FOUND);
}
