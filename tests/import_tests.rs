use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::TaskCreate;

fn app_state() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn import_json_inserts_all() {
    let repo = app_state();
    let payload = vec![
        TaskCreate {
            title: "a".into(),
            description: "d1".into(),
        },
        TaskCreate {
            title: "b".into(),
            description: "d2".into(),
        },
    ];

    let (code, Json(resp)) =
        rust_api_hub::handlers::task_handler::import_tasks_json(State(repo.clone()), Json(payload))
            .await;
    assert_eq!(code, StatusCode::CREATED);
    assert_eq!(resp["imported"].as_u64().unwrap(), 2);
    assert_eq!(repo.count(), 2);
}

#[tokio::test]
async fn import_csv_parses_and_inserts() {
    let repo = app_state();
    // CSV with header
    let csv = "title,description\nrow1,desc1\nrow2,desc2\n";
    let body = Bytes::from(csv);

    let (code, Json(resp)) =
        rust_api_hub::handlers::task_handler::import_tasks_csv(State(repo.clone()), body).await;
    assert_eq!(code, StatusCode::CREATED);
    assert_eq!(resp["imported"].as_u64().unwrap(), 2);
    assert_eq!(repo.count(), 2);
}

#[tokio::test]
async fn import_csv_bad_returns_400() {
    let repo = app_state();
    let bad = "not,a,csv\nthis is not valid rows";
    let body = Bytes::from(bad);
    let (code, _body) =
        rust_api_hub::handlers::task_handler::import_tasks_csv(State(repo.clone()), body).await;
    assert_eq!(code, StatusCode::BAD_REQUEST);
}
